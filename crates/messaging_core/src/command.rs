//! Durable per-plugin command inboxes built on Redis Streams.
//!
//! A caller `XADD`s a [`Message`] to `elupane:commands:<plugin_id>`. The target
//! plugin reads via a consumer group (`XREADGROUP`), executes, `XACK`s, and
//! optionally replies. Failed messages can be moved to the dead-letter stream.

use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::Result;
use redis::aio::MultiplexedConnection;
use redis::streams::{StreamId, StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, RedisResult};

use elupane_core::message::{keys, Message};

/// How long the consumer waits between polls when the command stream is idle.
///
/// Do **not** use `XREADGROUP ... BLOCK` here: redis-rs `MultiplexedConnection`
/// applies a default 500 ms response timeout to every command, so a blocking
/// read races that timeout — it errors with "timed out" and, worse, any command
/// Redis delivers after the client gave up is silently lost (it sits in the
/// consumer group's pending list but is never processed). Polling non-blocking
/// with a short sleep avoids both problems.
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// A message pulled from the command stream, with its Redis entry id.
#[derive(Debug, Clone)]
pub struct ConsumedMessage {
    pub stream_id: String,
    pub message: Message,
}

/// Pushes a command onto the target plugin's command stream.
pub async fn push_command(message: &Message) -> Result<()> {
    let stream = keys::command_stream(message.target.as_deref().unwrap_or(""));
    let mut con = cache::connection::conn().await?;
    let payload = serde_json::to_string(message)?;
    let _: Option<String> = con.xadd(&stream, "*", &[("payload", payload)]).await?;
    Ok(())
}

/// Consumes commands from a plugin's stream as a consumer-group member.
///
/// A stable consumer name (the plugin id) is used so pending work is reclaimed
/// after a restart via [`CommandConsumer::drain_pending`].
pub struct CommandConsumer {
    stream: String,
    group: String,
    consumer: String,
    conn: MultiplexedConnection,
}

impl CommandConsumer {
    /// Connect to the plugin's command stream, creating the consumer group
    /// (idempotently) if it does not exist yet.
    pub async fn connect(plugin_id: &str) -> Result<Self> {
        let stream = keys::command_stream(plugin_id);
        let group = plugin_id.to_string();
        let consumer = plugin_id.to_string();

        let mut conn = cache::connection::conn().await?;
        // Ignore BUSYGROUP (group already exists on restart).
        let _: RedisResult<()> = conn.xgroup_create_mkstream(&stream, &group, "0").await;

        Ok(Self {
            stream,
            group,
            consumer,
            conn,
        })
    }

    /// Reclaim this consumer's own pending (unacked) messages on startup.
    pub async fn drain_pending(&mut self) -> Result<Vec<ConsumedMessage>> {
        Ok(self.read(Some("0")).await?.unwrap_or_default())
    }

    /// Wait until the next command arrives for this plugin (polls the stream;
    /// never loses a delivered message to a client-side timeout).
    pub async fn next_message(&mut self) -> Result<ConsumedMessage> {
        loop {
            if let Some(mut batch) = self.read(Some(">")).await? {
                if !batch.is_empty() {
                    return Ok(batch.remove(0));
                }
            }
            // No new messages right now — poll again shortly instead of
            // busy-spinning.
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    /// Acknowledge a consumed message so it is removed from the group's PEL.
    pub async fn ack(&mut self, consumed: &ConsumedMessage) -> Result<()> {
        let _: usize = self
            .conn
            .xack(&self.stream, &self.group, &[consumed.stream_id.as_str()])
            .await?;
        Ok(())
    }

    /// Move a failed message to the dead-letter stream and acknowledge it so it
    /// stops redelivering.
    pub async fn dead_letter(&mut self, consumed: &ConsumedMessage, reason: &str) -> Result<()> {
        let mut map = BTreeMap::new();
        map.insert("message", serde_json::to_string(&consumed.message)?);
        map.insert("reason", reason.to_string());
        map.insert("stream_id", consumed.stream_id.clone());
        map.insert("original_stream", self.stream.clone());

        let _: Option<String> = self.conn.xadd_map(keys::DEAD_LETTER, "*", map).await?;
        self.ack(consumed).await?;
        Ok(())
    }

    async fn read(&mut self, id: Option<&str>) -> Result<Option<Vec<ConsumedMessage>>> {
        let opts = StreamReadOptions::default()
            .group(&self.group, &self.consumer)
            .count(8);

        let reply: Option<StreamReadReply> = self
            .conn
            .xread_options(&[self.stream.as_str()], &[id.unwrap_or(">")], &opts)
            .await?;

        let mut out = Vec::new();
        if let Some(reply) = reply {
            for stream_key in reply.keys {
                for id in stream_key.ids {
                    if let Ok(message) = decode(&id) {
                        out.push(ConsumedMessage {
                            stream_id: id.id.clone(),
                            message,
                        });
                    }
                }
            }
        }
        Ok(if out.is_empty() { None } else { Some(out) })
    }
}

/// Decode a [`Message`] from a stream entry's `payload` field.
fn decode(id: &StreamId) -> Result<Message> {
    let payload = id
        .map
        .get("payload")
        .and_then(|v| match v {
            redis::Value::BulkString(bytes) => Some(bytes.as_slice()),
            _ => None,
        })
        .ok_or_else(|| anyhow::anyhow!("stream entry missing payload"))?;

    Ok(serde_json::from_slice(payload)?)
}
