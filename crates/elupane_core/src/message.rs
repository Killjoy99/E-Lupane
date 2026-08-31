//! Message envelope and capability announcement types shared by every plugin.
//!
//! All inter-plugin traffic (commands, replies, domain events, capability
//! announcements) flows through this envelope, serialised to JSON and carried
//! over Redis (Pub/Sub for broadcasts/events, Streams for durable commands).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// The kind of a [`Message`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    /// A command sent to a specific plugin for execution (request/response).
    Command,
    /// A response to a command, correlated via `correlation_id`.
    Reply,
    /// A fire-and-forget domain event (no reply expected).
    Event,
    /// A capability/permission announcement broadcast on startup/heartbeat.
    CapabilityAnnouncement,
}

/// Structured error attached to a [`Message`] (typically a Reply).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageError {
    /// Machine-readable error code, e.g. `"auth.invalid_credentials"`.
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

impl MessageError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

/// The envelope for every message that crosses the Redis messaging layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message id.
    pub id: Uuid,
    pub kind: MessageKind,
    /// Operation name, e.g. `"user.create"`, `"auth.verify"`.
    pub op: String,
    /// Plugin id that produced the message.
    pub source: String,
    /// Plugin id (or reply channel) the message is aimed at, when relevant.
    #[serde(default)]
    pub target: Option<String>,
    /// Links a Reply back to the Command that triggered it.
    #[serde(default)]
    pub correlation_id: Option<Uuid>,
    /// Enables idempotent processing of at-least-once redeliveries.
    #[serde(default)]
    pub idempotency_key: Option<String>,
    /// The operation payload.
    #[serde(default)]
    pub payload: Value,
    /// Error detail (set on failed replies).
    #[serde(default)]
    pub error: Option<MessageError>,
    /// Unix timestamp (seconds).
    pub timestamp: i64,
}

impl Message {
    /// Build a command directed at a specific target plugin.
    pub fn command(op: &str, source: &str, target: &str, payload: Value) -> Self {
        Self::command_with_ids(op, source, target, payload, None, None)
    }

    /// Build a command with explicit correlation + idempotency ids.
    pub fn command_with_ids(
        op: &str,
        source: &str,
        target: &str,
        payload: Value,
        correlation_id: Option<Uuid>,
        idempotency_key: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: MessageKind::Command,
            op: op.to_string(),
            source: source.to_string(),
            target: Some(target.to_string()),
            correlation_id,
            idempotency_key,
            payload,
            error: None,
            timestamp: Utc::now().timestamp(),
        }
    }

    /// Build a successful reply to the given command.
    pub fn reply_to(command: &Message, source: &str, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: MessageKind::Reply,
            op: command.op.clone(),
            source: source.to_string(),
            target: Some(command.source.clone()),
            correlation_id: command.correlation_id,
            idempotency_key: command.idempotency_key.clone(),
            payload,
            error: None,
            timestamp: Utc::now().timestamp(),
        }
    }

    /// Build an error reply to the given command.
    pub fn error_reply(command: &Message, source: &str, code: &str, message: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: MessageKind::Reply,
            op: command.op.clone(),
            source: source.to_string(),
            target: Some(command.source.clone()),
            correlation_id: command.correlation_id,
            idempotency_key: command.idempotency_key.clone(),
            payload: Value::Null,
            error: Some(MessageError::new(code, message)),
            timestamp: Utc::now().timestamp(),
        }
    }

    /// Build a fire-and-forget domain event.
    pub fn event(op: &str, source: &str, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: MessageKind::Event,
            op: op.to_string(),
            source: source.to_string(),
            target: None,
            correlation_id: None,
            idempotency_key: None,
            payload,
            error: None,
            timestamp: Utc::now().timestamp(),
        }
    }

    /// True when this message carries an error.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

/// The announcement each plugin broadcasts on startup (and refreshes by
/// heartbeat) so the kernel and peers know what it can do.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAnnouncement {
    pub plugin_id: String,
    pub plugin_version: String,
    pub capabilities: Vec<String>,
    pub heartbeat_interval_secs: u64,
}

/// Well-known Redis keys / channels.
pub mod keys {
    use uuid::Uuid;

    /// Pub/Sub channel for capability announcements.
    pub const CAPABILITIES_CHANNEL: &str = "elupane:capabilities";
    /// Prefix for per-plugin durable capability registry keys
    /// (`elupane:registry:<plugin_id>`), each with its own TTL.
    pub const CAPABILITY_REGISTRY_PREFIX: &str = "elupane:registry:";
    /// Prefix for domain-event pub/sub channels.
    pub const EVENT_CHANNEL_PREFIX: &str = "elupane:events:";
    /// Prefix for per-plugin command streams.
    pub const COMMAND_STREAM_PREFIX: &str = "elupane:commands:";
    /// Prefix for per-request reply channels.
    pub const REPLY_CHANNEL_PREFIX: &str = "elupane:replies:";
    /// Dead-letter stream for messages that exhausted their retries.
    pub const DEAD_LETTER: &str = "elupane:deadletter";

    pub fn command_stream(plugin_id: &str) -> String {
        format!("{COMMAND_STREAM_PREFIX}{plugin_id}")
    }

    pub fn event_channel(domain: &str) -> String {
        format!("{EVENT_CHANNEL_PREFIX}{domain}")
    }

    pub fn reply_channel(correlation_id: Uuid) -> String {
        format!("{REPLY_CHANNEL_PREFIX}{correlation_id}")
    }

    pub fn capability_key(plugin_id: &str) -> String {
        format!("{CAPABILITY_REGISTRY_PREFIX}{plugin_id}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_serde_round_trip() {
        let msg = Message::command(
            "user.create",
            "gateway",
            "com.elupane.auth",
            serde_json::json!({ "username": "alice" }),
        );

        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();

        assert_eq!(back.kind, MessageKind::Command);
        assert_eq!(back.op, "user.create");
        assert_eq!(back.source, "gateway");
        assert_eq!(back.target.as_deref(), Some("com.elupane.auth"));
        assert_eq!(back.payload["username"], "alice");
        assert!(!back.is_error());
    }

    #[test]
    fn reply_links_to_command() {
        let correlation_id = Uuid::new_v4();
        let cmd = Message::command_with_ids(
            "auth.verify",
            "gateway",
            "com.elupane.auth",
            serde_json::json!({}),
            Some(correlation_id),
            Some("idem-1".into()),
        );

        let reply = Message::reply_to(&cmd, "com.elupane.auth", serde_json::json!({ "valid": true }));
        assert_eq!(reply.kind, MessageKind::Reply);
        assert_eq!(reply.correlation_id, Some(correlation_id));
        assert_eq!(reply.op, "auth.verify");

        let err = Message::error_reply(&cmd, "com.elupane.auth", "auth.error", "boom");
        assert!(err.is_error());
        assert_eq!(err.error.as_ref().unwrap().code, "auth.error");
    }
}

