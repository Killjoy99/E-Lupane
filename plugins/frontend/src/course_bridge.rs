//! Server-only bridge between the Dioxus fullstack server and the
//! course_management plugin. Performs request/response RPC against
//! `com.elupane.course_management` over the Redis messaging layer.

use std::time::Duration;

use anyhow::{anyhow, Result};
use serde_json::json;

use messaging_core::rpc::RequestClient;

use crate::course_types::{CourseInput, CourseView};

/// The course_management plugin's registered id.
pub const COURSE_PLUGIN: &str = "com.elupane.course_management";

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
    let reply = client().request(COURSE_PLUGIN, op, payload).await?;
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

/// List courses the caller may see. `mine` restricts to owned/managed courses.
pub async fn list_courses(
    token: &str,
    status: Option<String>,
    mine: bool,
    search: Option<String>,
) -> Result<Vec<CourseView>> {
    let mut map = serde_json::Map::new();
    if let Some(s) = status {
        map.insert("status".to_string(), json!(s));
    }
    map.insert("mine".to_string(), json!(mine));
    if let Some(q) = search {
        map.insert("search".to_string(), json!(q));
    }
    let value: serde_json::Value =
        request_typed("course.list", token_payload(token, map), "course_management plugin").await?;
    let list = value
        .get("courses")
        .cloned()
        .ok_or_else(|| anyhow!("course_management plugin reply missing 'courses'"))?;
    serde_json::from_value(list).map_err(|e| anyhow!("invalid courses list payload: {e}"))
}

pub async fn get_course(token: &str, course_id: uuid::Uuid) -> Result<CourseView> {
    request_typed(
        "course.get",
        token_payload(
            token,
            serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]),
        ),
        "course_management plugin",
    )
    .await
}

pub async fn create_course(token: &str, input: CourseInput) -> Result<CourseView> {
    let map = serde_json::to_value(input)?
        .as_object()
        .cloned()
        .unwrap_or_default();
    request_typed("course.create", token_payload(token, map), "course_management plugin").await
}

pub async fn update_course(
    token: &str,
    course_id: uuid::Uuid,
    input: CourseInput,
) -> Result<CourseView> {
    let mut map = serde_json::to_value(input)?.as_object().cloned().unwrap_or_default();
    map.insert("course_id".to_string(), json!(course_id));
    request_typed("course.update", token_payload(token, map), "course_management plugin").await
}

pub async fn delete_course(token: &str, course_id: uuid::Uuid) -> Result<()> {
    request_typed::<serde_json::Value>(
        "course.delete",
        token_payload(
            token,
            serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]),
        ),
        "course_management plugin",
    )
    .await?;
    Ok(())
}

/// Archive (soft-delete) a course.
pub async fn archive_course(token: &str, course_id: uuid::Uuid) -> Result<CourseView> {
    request_typed(
        "course.archive",
        token_payload(
            token,
            serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]),
        ),
        "course_management plugin",
    )
    .await
}

pub async fn duplicate_course(token: &str, course_id: uuid::Uuid) -> Result<CourseView> {
    request_typed(
        "course.duplicate",
        token_payload(
            token,
            serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]),
        ),
        "course_management plugin",
    )
    .await
}

pub async fn preview_course(token: &str, course_id: uuid::Uuid) -> Result<CourseView> {
    request_typed(
        "course.preview",
        token_payload(
            token,
            serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]),
        ),
        "course_management plugin",
    )
    .await
}

pub async fn set_status(token: &str, course_id: uuid::Uuid, status: &str) -> Result<CourseView> {
    request_typed(
        "course.set_status",
        token_payload(
            token,
            serde_json::Map::from_iter([
                ("course_id".to_string(), json!(course_id)),
                ("status".to_string(), json!(status)),
            ]),
        ),
        "course_management plugin",
    )
    .await
}

pub async fn publish_course(token: &str, course_id: uuid::Uuid) -> Result<CourseView> {
    request_typed(
        "course.publish",
        token_payload(
            token,
            serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]),
        ),
        "course_management plugin",
    )
    .await
}

pub async fn unpublish_course(token: &str, course_id: uuid::Uuid) -> Result<CourseView> {
    request_typed(
        "course.unpublish",
        token_payload(
            token,
            serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]),
        ),
        "course_management plugin",
    )
    .await
}

pub async fn submit_for_review(token: &str, course_id: uuid::Uuid) -> Result<CourseView> {
    request_typed(
        "course.submit_review",
        token_payload(
            token,
            serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]),
        ),
        "course_management plugin",
    )
    .await
}
