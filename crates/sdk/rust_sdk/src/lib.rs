//! E-Lupane Rust SDK — a small client that wraps the messaging core so a
//! plugin can connect, announce its capabilities, consume commands, and reply
//! in a few lines.
//!
//! ```no_run
//! # async fn example() -> anyhow::Result<()> {
//! let mut client = rust_sdk::PluginClient::connect("com.example.plugin").await?;
//! let _announcer = client.announce("1.0.0", vec!["example.read".into()]).await?;
//!
//! loop {
//!     let consumed = client.next_command().await?;
//!     let reply = serde_json::json!({ "ok": true });
//!     client.reply(&consumed.message, reply).await?;
//!     client.ack(&consumed).await?;
//! }
//! # }
//! ```

use anyhow::Result;
use serde_json::Value;

use cache;
use elupane_core::message::{keys, Message};
use messaging_core::capability::CapabilityAnnouncer;
use messaging_core::command::{CommandConsumer, ConsumedMessage};
use messaging_core::rpc::RequestClient;

/// Ready-to-use plugin messaging client.
pub struct PluginClient {
    plugin_id: String,
    consumer: CommandConsumer,
}

impl PluginClient {
    /// Connect to the plugin's command stream.
    ///
    /// Redis must already be initialised (`cache::connection::connect`).
    pub async fn connect(plugin_id: &str) -> Result<Self> {
        let consumer = CommandConsumer::connect(plugin_id).await?;
        Ok(Self {
            plugin_id: plugin_id.to_string(),
            consumer,
        })
    }

    /// Broadcast this plugin's capabilities (Pub/Sub + durable registry) and
    /// return an announcer for heartbeats.
    pub async fn announce(
        &self,
        version: &str,
        capabilities: Vec<String>,
    ) -> Result<CapabilityAnnouncer> {
        let announcer = CapabilityAnnouncer::new(&self.plugin_id, version, capabilities, 10);
        announcer.announce().await?;
        Ok(announcer)
    }

    /// Reclaim this consumer's own pending (unacked) commands on startup.
    pub async fn drain_pending(&mut self) -> Result<Vec<ConsumedMessage>> {
        self.consumer.drain_pending().await
    }

    /// Block until the next command arrives for this plugin.
    pub async fn next_command(&mut self) -> Result<ConsumedMessage> {
        self.consumer.next_message().await
    }

    /// Acknowledge a consumed command.
    pub async fn ack(&mut self, consumed: &ConsumedMessage) -> Result<()> {
        self.consumer.ack(consumed).await
    }

    /// Move a failed command to the dead-letter stream.
    pub async fn dead_letter(&mut self, consumed: &ConsumedMessage, reason: &str) -> Result<()> {
        self.consumer.dead_letter(consumed, reason).await
    }

    /// Build a client for making request/response calls to other plugins.
    pub fn rpc(&self) -> RequestClient {
        RequestClient::new(&self.plugin_id)
    }

    /// Publish a successful reply to a consumed command.
    pub async fn reply(&self, command: &Message, payload: Value) -> Result<()> {
        let reply = Message::reply_to(command, &self.plugin_id, payload);
        self.send_reply(&reply).await
    }

    /// Publish an error reply to a consumed command.
    pub async fn reply_error(&self, command: &Message, code: &str, message: &str) -> Result<()> {
        let reply = Message::error_reply(command, &self.plugin_id, code, message);
        self.send_reply(&reply).await
    }

    /// Publish a pre-built reply message to its correlation channel.
    pub async fn send_reply(&self, reply: &Message) -> Result<()> {
        let correlation_id = reply
            .correlation_id
            .ok_or_else(|| anyhow::anyhow!("reply message has no correlation id"))?;
        let channel = keys::reply_channel(correlation_id);
        let payload = serde_json::to_vec(reply)?;
        cache::pubsub::publish(&channel, payload).await
    }
}

