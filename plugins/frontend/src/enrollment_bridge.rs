//! Server-only bridge between the Dioxus fullstack server and the
//! enrollments plugin. Performs request/response RPC against
//! `com.elupane.enrollments` over the Redis messaging layer.

use std::time::Duration;

use anyhow::{anyhow, Result};
use serde_json::json;

use messaging_core::rpc::RequestClient;

use crate::enrollment_types::{
    AccessView, BulkEnrollView, EnrollmentView, HistoryView, InviteView,
};

/// The enrollments plugin's registered id.
pub const ENROLLMENT_PLUGIN: &str = "com.elupane.enrollments";

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
    let reply = client().request(ENROLLMENT_PLUGIN, op, payload).await?;
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

async fn extract_list(value: serde_json::Value, key: &str) -> Result<serde_json::Value> {
    value
        .get(key)
        .cloned()
        .ok_or_else(|| anyhow!("enrollments plugin reply missing '{key}'"))
}

/// Student self-enrollment (lands in `pending`).
pub async fn self_enroll(token: &str, course_id: uuid::Uuid) -> Result<EnrollmentView> {
    request_typed(
        "enrollment.self",
        map1(token, "course_id", json!(course_id)),
        "enrollments plugin",
    )
    .await
}

/// Instructor/admin enrolls a student by username or email.
pub async fn enroll_student(
    token: &str,
    course_id: uuid::Uuid,
    student: &str,
    expires_at: Option<String>,
) -> Result<EnrollmentView> {
    let mut map = serde_json::Map::from_iter([
        ("course_id".to_string(), json!(course_id)),
        ("student".to_string(), json!(student)),
    ]);
    if let Some(exp) = expires_at {
        map.insert("expires_at".to_string(), json!(exp));
    }
    request_typed("enrollment.enroll", token_payload(token, map), "enrollments plugin").await
}

/// Bulk enroll many students at once.
pub async fn bulk_enroll(
    token: &str,
    course_id: uuid::Uuid,
    students: Vec<String>,
    expires_at: Option<String>,
) -> Result<BulkEnrollView> {
    let mut map = serde_json::Map::from_iter([
        ("course_id".to_string(), json!(course_id)),
        ("students".to_string(), json!(students)),
    ]);
    if let Some(exp) = expires_at {
        map.insert("expires_at".to_string(), json!(exp));
    }
    request_typed("enrollment.bulk", token_payload(token, map), "enrollments plugin").await
}

/// Change an enrollment's state (approve/suspend/cancel/complete/reenroll).
pub async fn change_state(
    token: &str,
    action: &str,
    enrollment_id: uuid::Uuid,
    reason: Option<String>,
) -> Result<EnrollmentView> {
    let mut map = serde_json::Map::from_iter([("enrollment_id".to_string(), json!(enrollment_id))]);
    if let Some(r) = reason {
        map.insert("reason".to_string(), json!(r));
    }
    request_typed(
        &format!("enrollment.{action}"),
        token_payload(token, map),
        "enrollments plugin",
    )
    .await
}

/// Roster of a course (managers) — optionally filtered by state.
pub async fn roster(
    token: &str,
    course_id: uuid::Uuid,
    state: Option<String>,
) -> Result<Vec<EnrollmentView>> {
    let mut map = serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]);
    if let Some(s) = state {
        map.insert("state".to_string(), json!(s));
    }
    let value: serde_json::Value =
        request_typed("enrollment.roster", token_payload(token, map), "enrollments plugin").await?;
    serde_json::from_value(extract_list(value, "enrollments").await?)
        .map_err(|e| anyhow!("invalid roster payload: {e}"))
}

/// The caller's own enrollments.
pub async fn my_enrollments(token: &str) -> Result<Vec<EnrollmentView>> {
    let value: serde_json::Value = request_typed(
        "enrollment.mine",
        map1(token, "token", json!(token)),
        "enrollments plugin",
    )
    .await?;
    serde_json::from_value(extract_list(value, "enrollments").await?)
        .map_err(|e| anyhow!("invalid enrollments payload: {e}"))
}

/// Enrollment history (audit trail) for one enrollment.
pub async fn history(token: &str, enrollment_id: uuid::Uuid) -> Result<Vec<HistoryView>> {
    let value: serde_json::Value = request_typed(
        "enrollment.history",
        map1(token, "enrollment_id", json!(enrollment_id)),
        "enrollments plugin",
    )
    .await?;
    serde_json::from_value(extract_list(value, "history").await?)
        .map_err(|e| anyhow!("invalid history payload: {e}"))
}

/// Whether the caller may access a course's content.
pub async fn access(token: &str, course_id: uuid::Uuid) -> Result<AccessView> {
    request_typed(
        "enrollment.access",
        map1(token, "course_id", json!(course_id)),
        "enrollments plugin",
    )
    .await
}

/// Create an invite (course link) for a course.
pub async fn create_invite(
    token: &str,
    course_id: uuid::Uuid,
    expires_at: Option<String>,
    max_uses: Option<i64>,
) -> Result<InviteView> {
    let mut map = serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]);
    if let Some(exp) = expires_at {
        map.insert("expires_at".to_string(), json!(exp));
    }
    if let Some(mu) = max_uses {
        map.insert("max_uses".to_string(), json!(mu));
    }
    request_typed("enrollment.invite.create", token_payload(token, map), "enrollments plugin").await
}

/// List a course's invites (managers).
pub async fn list_invites(token: &str, course_id: uuid::Uuid) -> Result<Vec<InviteView>> {
    let value: serde_json::Value = request_typed(
        "enrollment.invite.list",
        map1(token, "course_id", json!(course_id)),
        "enrollments plugin",
    )
    .await?;
    serde_json::from_value(extract_list(value, "invites").await?)
        .map_err(|e| anyhow!("invalid invites payload: {e}"))
}

/// Revoke an invite.
pub async fn revoke_invite(token: &str, invite_id: uuid::Uuid) -> Result<InviteView> {
    request_typed(
        "enrollment.invite.revoke",
        map1(token, "invite_id", json!(invite_id)),
        "enrollments plugin",
    )
    .await
}

/// Join a course via an invite token (the "course link" flow).
pub async fn join(token: &str, invite_token: &str) -> Result<EnrollmentView> {
    request_typed(
        "enrollment.join",
        map1(token, "token", json!(invite_token)),
        "enrollments plugin",
    )
    .await
}