//! Redis Pub/Sub helpers.
//!
//! Used for capability announcements and fire-and-forget domain events.
//! Pub/Sub is intentionally fire-and-forget: use the messaging crate's
//! streams (consumer groups) for anything that must be delivered.

use anyhow::Result;
use futures_util::{Stream, StreamExt};
use redis::aio::PubSub;
use redis::AsyncCommands;

use crate::connection::client;

/// Subscribe to a channel, returning a stream of decoded string payloads.
///
/// A dedicated connection is created per subscription (Redis requires it).
pub async fn subscribe(channel: impl Into<String>) -> Result<impl Stream<Item = String>> {
    let channel = channel.into();
    let mut pubsub: PubSub = client().get_async_pubsub().await?;
    pubsub.subscribe(&channel).await?;

    let stream = pubsub
        .into_on_message()
        .filter_map(|msg| async move {
            Some(
                String::from_utf8_lossy(&msg.get_payload::<Vec<u8>>().unwrap_or_default())
                    .into_owned(),
            )
        });

    Ok(stream)
}

/// Publish a payload to a channel.
pub async fn publish(channel: &str, payload: impl AsRef<[u8]>) -> Result<()> {
    let mut con = crate::connection::conn().await?;
    let _: () = con.publish(channel, payload.as_ref()).await?;
    Ok(())
}

