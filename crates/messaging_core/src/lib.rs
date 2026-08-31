//! High-level inter-plugin messaging over Redis.
//!
//! - `command`  — durable per-plugin command inboxes (Redis Streams + consumer
//!   groups): read, ack, dead-letter.
//! - `rpc`      — request/response with correlation ids, reply channels and
//!   timeouts.
//! - `capability` — broadcast / registry of plugin permissions.
//! - `events`   — fire-and-forget domain events on Pub/Sub.
//! - `idempotency` — at-least-once redelivery guard.

pub mod capability;
pub mod command;
pub mod events;
pub mod idempotency;
pub mod rpc;

pub use capability::{CapabilityAnnouncer, list_capabilities, subscribe_capabilities};
pub use command::{CommandConsumer, ConsumedMessage, push_command};
pub use events::{publish_event, subscribe_events};
pub use idempotency::IdempotencyGuard;
pub use rpc::RequestClient;
