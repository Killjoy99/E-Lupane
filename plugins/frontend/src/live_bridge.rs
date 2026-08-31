//! Server-only bridge between the Dioxus fullstack server and the
//! live_classes plugin. Performs request/response RPC against
//! `com.elupane.live_classes` over the Redis messaging layer for durable
//! state (scheduling, attendance, chat history, resources). Real-time
//! transport for `webrtc`-provider sessions (video, screen share, chat
//! relay, whiteboard) is handled separately by
//! [`crate::live_signaling`]'s WebSocket endpoint.

use std::time::Duration;

use anyhow::{anyhow, Result};
use serde_json::json;
use uuid::Uuid;

use messaging_core::rpc::RequestClient;

use crate::live_types::{
    LiveAttendanceView, LiveChatMessageView, LiveJoinView, LiveResourceView, LiveSessionView,
};

/// The live_classes plugin's registered id.
pub const LIVE_CLASSES_PLUGIN: &str = "com.elupane.live_classes";

/// Who we are on the messaging layer.
const SOURCE: &str = "com.elupane.frontend";

fn client() -> RequestClient {
    RequestClient::new(SOURCE).with_timeout(Duration::from_secs(8))
}

async fn request_typed<T: serde::de::DeserializeOwned>(
    op: &str,
    payload: serde_json::Value,
    what: &str,
) -> Result<T> {
    let reply = client().request(LIVE_CLASSES_PLUGIN, op, payload).await?;
    if let Some(err) = &reply.error {
        return Err(anyhow!("{}: {}", what, err.message));
    }
    serde_json::from_value(reply.payload)
        .map_err(|e| anyhow!("{} returned an unexpected payload: {e}", what))
}

fn token_payload(token: &str, mut extra: serde_json::Map<String, serde_json::Value>) -> serde_json::Value {
    extra.insert("token".to_string(), json!(token));
    serde_json::Value::Object(extra)
}

fn map1(token: &str, k: &str, v: serde_json::Value) -> serde_json::Value {
    token_payload(token, serde_json::Map::from_iter([(k.to_string(), v)]))
}

async fn extract_list(value: serde_json::Value, key: &str) -> Result<serde_json::Value> {
    value
        .get(key)
        .cloned()
        .ok_or_else(|| anyhow!("live_classes plugin reply missing '{key}'"))
}

/// Schedule a new live session for a course (instructor/admin).
#[allow(clippy::too_many_arguments)]
pub async fn schedule(
    token: &str,
    course_id: Uuid,
    title: &str,
    description: &str,
    scheduled_start: String,
    scheduled_end: String,
    provider: &str,
    external_url: Option<String>,
) -> Result<LiveSessionView> {
    let mut map = serde_json::Map::from_iter([
        ("course_id".to_string(), json!(course_id)),
        ("title".to_string(), json!(title)),
        ("description".to_string(), json!(description)),
        ("scheduled_start".to_string(), json!(scheduled_start)),
        ("scheduled_end".to_string(), json!(scheduled_end)),
        ("provider".to_string(), json!(provider)),
    ]);
    if let Some(url) = external_url {
        map.insert("external_url".to_string(), json!(url));
    }
    request_typed("live.schedule", token_payload(token, map), "live_classes plugin").await
}

/// List a course's sessions (upcoming, live and past).
pub async fn list_for_course(token: &str, course_id: Uuid) -> Result<Vec<LiveSessionView>> {
    let value: serde_json::Value =
        request_typed("live.list", map1(token, "course_id", json!(course_id)), "live_classes plugin")
            .await?;
    serde_json::from_value(extract_list(value, "sessions").await?)
        .map_err(|e| anyhow!("invalid sessions payload: {e}"))
}

/// A single session's details.
pub async fn get(token: &str, session_id: Uuid) -> Result<LiveSessionView> {
    request_typed("live.get", map1(token, "session_id", json!(session_id)), "live_classes plugin").await
}

/// Join a session: validates access, records attendance, returns the
/// WebSocket signaling path to connect to (webrtc provider).
pub async fn join(token: &str, session_id: Uuid) -> Result<LiveJoinView> {
    request_typed("live.join", map1(token, "session_id", json!(session_id)), "live_classes plugin").await
}

/// Record that the caller left the session (stamps attendance duration).
pub async fn leave(token: &str, attendance_id: Uuid) -> Result<()> {
    let _: serde_json::Value = request_typed(
        "live.leave",
        map1(token, "attendance_id", json!(attendance_id)),
        "live_classes plugin",
    )
    .await?;
    Ok(())
}

/// Start a scheduled session (host/instructor/admin).
pub async fn start(token: &str, session_id: Uuid) -> Result<LiveSessionView> {
    request_typed("live.start", map1(token, "session_id", json!(session_id)), "live_classes plugin").await
}

/// End a live session.
pub async fn end(token: &str, session_id: Uuid) -> Result<LiveSessionView> {
    request_typed("live.end", map1(token, "session_id", json!(session_id)), "live_classes plugin").await
}

/// Cancel a scheduled session.
pub async fn cancel(token: &str, session_id: Uuid) -> Result<LiveSessionView> {
    request_typed("live.cancel", map1(token, "session_id", json!(session_id)), "live_classes plugin").await
}

/// Attendance roster for a session (managers only).
pub async fn attendance(token: &str, session_id: Uuid) -> Result<Vec<LiveAttendanceView>> {
    let value: serde_json::Value = request_typed(
        "live.attendance.list",
        map1(token, "session_id", json!(session_id)),
        "live_classes plugin",
    )
    .await?;
    serde_json::from_value(extract_list(value, "attendance").await?)
        .map_err(|e| anyhow!("invalid attendance payload: {e}"))
}

/// Persisted chat history for a session.
pub async fn chat_history(token: &str, session_id: Uuid) -> Result<Vec<LiveChatMessageView>> {
    let value: serde_json::Value = request_typed(
        "live.chat.history",
        map1(token, "session_id", json!(session_id)),
        "live_classes plugin",
    )
    .await?;
    serde_json::from_value(extract_list(value, "messages").await?)
        .map_err(|e| anyhow!("invalid chat history payload: {e}"))
}

/// Send + persist a chat message (also broadcast live via the signaling
/// WebSocket by the caller — this only durably records it).
pub async fn send_chat(token: &str, session_id: Uuid, body: &str) -> Result<LiveChatMessageView> {
    let map = serde_json::Map::from_iter([
        ("session_id".to_string(), json!(session_id)),
        ("body".to_string(), json!(body)),
    ]);
    request_typed("live.chat.send", token_payload(token, map), "live_classes plugin").await
}

/// Attach a resource (link, whiteboard snapshot, recording pointer) to a session.
pub async fn attach_resource(
    token: &str,
    session_id: Uuid,
    name: &str,
    url: &str,
    kind: &str,
) -> Result<LiveResourceView> {
    let map = serde_json::Map::from_iter([
        ("session_id".to_string(), json!(session_id)),
        ("name".to_string(), json!(name)),
        ("url".to_string(), json!(url)),
        ("kind".to_string(), json!(kind)),
    ]);
    request_typed("live.resource.attach", token_payload(token, map), "live_classes plugin").await
}

/// List a session's resources.
pub async fn list_resources(token: &str, session_id: Uuid) -> Result<Vec<LiveResourceView>> {
    let value: serde_json::Value = request_typed(
        "live.resource.list",
        map1(token, "session_id", json!(session_id)),
        "live_classes plugin",
    )
    .await?;
    serde_json::from_value(extract_list(value, "resources").await?)
        .map_err(|e| anyhow!("invalid resources payload: {e}"))
}
