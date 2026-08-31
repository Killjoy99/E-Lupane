use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use anyhow::Result;
use futures_util::StreamExt;

use cache;
use config::AppConfig;
use elupane_core::message::CapabilityAnnouncement;
use event_bus::EventBus;
use messaging_core;
use plugin_manager::{
    PluginManager,
    launcher::launch_plugin,
    discovery::discover_plugins,
};
use tokio::sync::RwLock;

use crate::state::AppState;

pub async fn bootstrap() -> Result<AppState> {
    let config = AppConfig::load().expect("Failed to load application config");

    info!(
        "Starting {} in {} environment",
        config.app_name,
        config.environment
    );

    // Redis first — every plugin talks to every other plugin through it.
    cache::connection::connect(&config.redis_url).await?;
    info!("Connected to Redis at {}", config.redis_url);

    let event_bus = Arc::new(EventBus::new());

    // Live capability registry: seeded from the durable Redis keys, then kept
    // up to date from Pub/Sub announcements.
    let capabilities: Arc<RwLock<HashMap<String, CapabilityAnnouncement>>> =
        Arc::new(RwLock::new(HashMap::new()));

    if let Ok(existing) = messaging_core::list_capabilities().await {
        let mut map = capabilities.write().await;
        for announcement in existing {
            map.insert(announcement.plugin_id.clone(), announcement);
        }
    }

    {
        let registry = capabilities.clone();
        tokio::spawn(async move {
            let mut stream = match messaging_core::subscribe_capabilities().await {
                Ok(stream) => Box::pin(stream),
                Err(err) => {
                    tracing::error!("failed to subscribe to capability announcements: {err}");
                    return;
                }
            };

            while let Some(announcement) = stream.next().await {
                tracing::info!(
                    "capability announcement: {} v{} ({})",
                    announcement.plugin_id,
                    announcement.plugin_version,
                    announcement.capabilities.len()
                );
                registry
                    .write()
                    .await
                    .insert(announcement.plugin_id.clone(), announcement);
            }
        });
    }

    // initialise the plugin manager and discover plugins
    let plugin_manager = Arc::new(PluginManager::new());

    let manifests = discover_plugins("plugins")?;

    // Collect child handles so the supervisor can manage their lifecycle.
    let mut children = Vec::new();

    for manifest in manifests {
        info!("Discovered plugin: {} v{}", manifest.name, manifest.version);

        match launch_plugin(&manifest).await {
            Ok((pid, child)) => {
                info!("Started plugin '{}' with PID {}", manifest.name, pid);
                children.push(child);
            }
            Err(err) => {
                // One missing/broken plugin must not take the whole gateway
                // down — the rest of the system keeps moving.
                tracing::error!(
                    "Failed to start plugin '{}': {err} — continuing with the remaining plugins",
                    manifest.name
                );
            }
        }
    }

    Ok(AppState {
        plugin_manager,
        event_bus,
        plugin_children: children,
        capabilities,
    })
}

