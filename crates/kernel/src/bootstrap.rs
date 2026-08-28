use std::sync::Arc;
use tracing::info;

use anyhow::Result;

use config::AppConfig;
use event_bus::EventBus;
use plugin_manager::{
    PluginManager,
    launcher::launch_plugin,
    discovery::discover_plugins,
};

use crate::state::AppState;

pub async fn bootstrap() -> Result<AppState> {
    let config = AppConfig::load().expect("Failed to load application config");

    info!(
        "Starting {} in {} environment",
        config.app_name,
        config.environment
    );

    // initialise the event bus
    let event_bus = Arc::new(EventBus::new(100));
    // initialise the plugin manager and discover plugins
    let plugin_manager = Arc::new(PluginManager::new());

    let manifests = discover_plugins("plugins")?;

    // Collect child handles so the supervisor can manage their lifecycle.
    let mut children = Vec::new();

    for manifest in manifests {
        info!("Discovered plugin: {} v{}", manifest.name, manifest.version);

        let (pid, child) = launch_plugin(&manifest).await?;
        info!("Started plugin '{}' with PID {}", manifest.name, pid);
        children.push(child);
    }

    Ok(AppState {
        plugin_manager,
        event_bus,
        plugin_children: children,
    })
}
