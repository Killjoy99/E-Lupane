//! Redis-backed event bus.
//!
//! System and domain events are serialised to JSON and broadcast on Redis
//! Pub/Sub (`elupane:events:system`). This connects the plugin *processes*,
//! unlike the old in-process `tokio::broadcast` channel.

use anyhow::Result;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};

/// Well-known system event channel.
pub const SYSTEM_CHANNEL: &str = "elupane:events:system";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SystemEvent {
    PluginLoaded(String),
    PluginEnabled(String),
    PluginDisabled(String),
}

/// A thin typed wrapper over the shared Redis client.
#[derive(Clone, Copy, Default)]
pub struct EventBus;

impl EventBus {
    pub fn new() -> Self {
        Self
    }

    /// Publish a system event to all subscribers.
    pub async fn publish(&self, event: SystemEvent) -> Result<()> {
        let payload = serde_json::to_vec(&event)?;
        cache::pubsub::publish(SYSTEM_CHANNEL, payload).await
    }

    /// Subscribe to system events (dedicated Pub/Sub connection).
    pub async fn subscribe(&self) -> Result<impl Stream<Item = SystemEvent>> {
        let stream = cache::pubsub::subscribe(SYSTEM_CHANNEL).await?;
        Ok(stream.filter_map(|raw| async move {
            serde_json::from_str::<SystemEvent>(&raw).ok()
        }))
    }
}
