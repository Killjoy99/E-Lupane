//! Request/response RPC over Redis.
//!
//! The caller generates a correlation id, subscribes to a per-request reply
//! channel, `XADD`s a command to the target plugin's stream, then waits
//! (with a timeout) for the plugin to publish its reply.

use std::time::Duration;

use anyhow::Result;
use futures_util::StreamExt;
use redis::aio::PubSub;
use serde_json::Value;
use uuid::Uuid;

use crate::command::push_command;
use elupane_core::message::{keys, Message};

/// Default RPC timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Sends commands and awaits the matching reply.
#[derive(Clone)]
pub struct RequestClient {
    source: String,
    timeout: Duration,
}

impl RequestClient {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send a command to `target` and await the reply (result or error).
    pub async fn request(&self, target: &str, op: &str, payload: Value) -> Result<Message> {
        let correlation_id = Uuid::new_v4();
        let idempotency_key = Uuid::new_v4().to_string();

        let command = Message::command_with_ids(
            op,
            &self.source,
            target,
            payload,
            Some(correlation_id),
            Some(idempotency_key),
        );

        // Subscribe to the reply channel BEFORE publishing the command so no
        // reply can be missed.
        let mut pubsub: PubSub = cache::connection::client().get_async_pubsub().await?;
        let reply_channel = keys::reply_channel(correlation_id);
        pubsub.subscribe(&reply_channel).await?;

        push_command(&command).await?;

        let reply_fut = async {
            let mut stream = pubsub.on_message();
            loop {
                match stream.next().await {
                    Some(msg) => {
                        let raw = msg.get_payload::<Vec<u8>>().unwrap_or_default();
                        if let Ok(reply) = serde_json::from_slice::<Message>(&raw) {
                            break Ok(reply);
                        }
                    }
                    None => break Err(anyhow::anyhow!("reply channel closed")),
                }
            }
        };

        match tokio::time::timeout(self.timeout, reply_fut).await {
            Ok(result) => result,
            Err(_) => Err(anyhow::anyhow!(
                "request '{op}' to '{target}' timed out after {:?}",
                self.timeout
            )),
        }
    }

    /// Fire a command without waiting for a reply.
    pub async fn send(&self, target: &str, op: &str, payload: Value) -> Result<()> {
        let command = Message::command(op, &self.source, target, payload);
        push_command(&command).await
    }
}
