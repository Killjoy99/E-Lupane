//! Server-only bridge between the Dioxus fullstack server and the
//! messaging plugin. Performs request/response RPC against
//! `com.elupane.messaging` over the Redis messaging layer.

use std::time::Duration;

use anyhow::{anyhow, Result};
use serde_json::json;

use messaging_core::rpc::RequestClient;

use crate::messaging_types::{
    CreateReplyInput, CreateTopicInput, DiscussionReplyView, DiscussionTopicView,
};

pub const MESSAGING_PLUGIN: &str = "com.elupane.messaging";
const SOURCE: &str = "com.elupane.frontend";

fn client() -> RequestClient {
    RequestClient::new(SOURCE).with_timeout(Duration::from_secs(8))
}

async fn request_typed<T: serde::de::DeserializeOwned>(
    op: &str,
    payload: serde_json::Value,
    what: &str,
) -> Result<T> {
    let reply = client().request(MESSAGING_PLUGIN, op, payload).await?;
    if let Some(err) = &reply.error {
        return Err(anyhow!("{}: {}", what, err.message));
    }
    serde_json::from_value(reply.payload)
        .map_err(|e| anyhow!("{} returned an unexpected payload: {e}", what))
}

fn token_payload(
    token: &str,
    mut extra: serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    extra.insert("token".to_string(), json!(token));
    serde_json::Value::Object(extra)
}

fn map1(token: &str, k: &str, v: serde_json::Value) -> serde_json::Value {
    token_payload(token, serde_json::Map::from_iter([(k.to_string(), v)]))
}

pub async fn list_topics(token: &str, course_id: uuid::Uuid) -> Result<Vec<DiscussionTopicView>> {
    request_typed(
        "messaging.discussion.list",
        map1(token, "course_id", json!(course_id)),
        "messaging plugin",
    )
    .await
}

pub async fn create_topic(
    token: &str,
    input: CreateTopicInput,
) -> Result<DiscussionTopicView> {
    let mut map = serde_json::to_value(input)?;
    map.as_object_mut()
        .unwrap()
        .insert("token".to_string(), json!(token));
    request_typed("messaging.discussion.create", map, "messaging plugin").await
}

pub async fn create_reply(
    token: &str,
    input: CreateReplyInput,
) -> Result<DiscussionReplyView> {
    let mut map = serde_json::to_value(input)?;
    map.as_object_mut()
        .unwrap()
        .insert("token".to_string(), json!(token));
    request_typed("messaging.discussion.reply", map, "messaging plugin").await
}

pub async fn list_replies(
    token: &str,
    topic_id: uuid::Uuid,
) -> Result<Vec<DiscussionReplyView>> {
    let value: serde_json::Value = request_typed(
        "messaging.discussion.list_replies",
        map1(token, "id", json!(topic_id)),
        "messaging plugin",
    )
    .await?;
    serde_json::from_value(value).map_err(|e| anyhow!("invalid replies payload: {e}"))
}
