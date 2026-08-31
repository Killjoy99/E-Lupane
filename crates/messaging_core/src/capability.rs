//! Capability (permission) broadcasting and registry.
//!
//! Each plugin announces its permission set on startup by:
//! 1. Publishing a `CapabilityAnnouncement` on the Pub/Sub
//!    `elupane:capabilities` channel (for live subscribers), and
//! 2. Writing `elupane:registry:<plugin_id>` with a TTL, refreshed by
//!    heartbeats, so the kernel can enumerate who is alive and what they do.

use anyhow::Result;
use futures_util::{Stream, StreamExt};
use redis::AsyncCommands;

use elupane_core::message::{keys, CapabilityAnnouncement};

/// Announces (and keeps alive) a plugin's capabilities.
#[derive(Clone)]
pub struct CapabilityAnnouncer {
    announcement: CapabilityAnnouncement,
    /// Registry key TTL = heartbeat interval × 3.
    ttl_secs: u64,
}

impl CapabilityAnnouncer {
    pub fn new(
        plugin_id: &str,
        plugin_version: &str,
        capabilities: Vec<String>,
        heartbeat_interval_secs: u64,
    ) -> Self {
        let ttl_secs = heartbeat_interval_secs.saturating_mul(3).max(1);
        Self {
            announcement: CapabilityAnnouncement {
                plugin_id: plugin_id.to_string(),
                plugin_version: plugin_version.to_string(),
                capabilities,
                heartbeat_interval_secs,
            },
            ttl_secs,
        }
    }

    /// Broadcast the announcement and write the durable registry entry.
    pub async fn announce(&self) -> Result<()> {
        let payload = serde_json::to_vec(&self.announcement)?;
        cache::pubsub::publish(keys::CAPABILITIES_CHANNEL, payload).await?;
        self.refresh_registry().await
    }

    /// Refresh the durable registry entry's TTL (heartbeat tick).
    pub async fn heartbeat(&self) -> Result<()> {
        self.refresh_registry().await
    }

    async fn refresh_registry(&self) -> Result<()> {
        let mut con = cache::connection::conn().await?;
        let entry = serde_json::to_string(&self.announcement)?;
        let key = keys::capability_key(&self.announcement.plugin_id);
        let _: () = con
            .set_options(
                key,
                entry,
                redis::SetOptions::default().with_expiration(redis::SetExpiry::EX(self.ttl_secs)),
            )
            .await?;
        Ok(())
    }
}

/// Subscribe to capability announcements broadcast by plugins.
pub async fn subscribe_capabilities(
) -> Result<impl Stream<Item = CapabilityAnnouncement>> {
    let stream = cache::pubsub::subscribe(keys::CAPABILITIES_CHANNEL).await?;
    Ok(stream.filter_map(|raw| async move {
        serde_json::from_str::<CapabilityAnnouncement>(&raw).ok()
    }))
}

/// Enumerate the live capability registry (`elupane:registry:*` keys).
pub async fn list_capabilities() -> Result<Vec<CapabilityAnnouncement>> {
    let mut con = cache::connection::conn().await?;

    let mut iter: redis::AsyncIter<String> = con
        .scan_match(format!("{}*", keys::CAPABILITY_REGISTRY_PREFIX))
        .await?;

    let mut keys = Vec::new();
    while let Some(key) = iter.next_item().await {
        if let Ok(key) = key {
            keys.push(key);
        }
    }
    // Release the scan iterator's borrow on `con` before issuing GETs.
    drop(iter);

    let mut announcements = Vec::new();
    for key in keys {
        let raw: Option<String> = con.get(&key).await?;
        if let Some(raw) = raw {
            if let Ok(a) = serde_json::from_str::<CapabilityAnnouncement>(&raw) {
                announcements.push(a);
            }
        }
    }
    Ok(announcements)
}
