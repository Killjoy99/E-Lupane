//! Fire-and-forget domain events on Redis Pub/Sub.
//!
//! Events are broadcast on `elupane:events:<domain>` and are intentionally
//! non-durable: subscribers react or the event is gone. Use commands (streams)
//! for anything that must be delivered.

use anyhow::Result;
use futures_util::{Stream, StreamExt};

use elupane_core::message::{keys, Message};

/// Publish a domain event (no reply expected).
pub async fn publish_event(domain: &str, event: &Message) -> Result<()> {
    let channel = keys::event_channel(domain);
    let payload = serde_json::to_vec(event)?;
    cache::pubsub::publish(&channel, payload).await
}

/// Subscribe to domain events on a channel.
pub async fn subscribe_events(domain: &str) -> Result<impl Stream<Item = Message>> {
    let channel = keys::event_channel(domain);
    let stream = cache::pubsub::subscribe(channel).await?;
    Ok(stream.filter_map(|raw| async move {
        serde_json::from_str::<Message>(&raw).ok()
    }))
}
