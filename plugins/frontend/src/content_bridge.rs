//! Server-only bridge between the Dioxus fullstack server and the
//! content plugin. Performs request/response RPC against
//! `com.elupane.content` over the Redis messaging layer.

use std::time::Duration;

use anyhow::{anyhow, Result};
use serde_json::json;

use messaging_core::rpc::RequestClient;

use crate::content_types::{
    ContentFileView, ContentItemView, ContentLessonView, ContentModuleView, CreateContentItemInput,
    CreateLessonInput, CreateModuleInput, FileDownloadView, UpdateContentItemInput, UpdateLessonInput,
    UpdateModuleInput,
};

/// The content plugin's registered id.
pub const CONTENT_PLUGIN: &str = "com.elupane.content";

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
    let reply = client().request(CONTENT_PLUGIN, op, payload).await?;
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

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

pub async fn list_modules(
    token: &str,
    course_id: uuid::Uuid,
) -> Result<Vec<ContentModuleView>> {
    request_typed(
        "content.module.list",
        token_payload(
            token,
            serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]),
        ),
        "content plugin",
    )
    .await
}

pub async fn get_module(token: &str, module_id: uuid::Uuid) -> Result<ContentModuleView> {
    request_typed(
        "content.module.get",
        token_payload(
            token,
            serde_json::Map::from_iter([("module_id".to_string(), json!(module_id))]),
        ),
        "content plugin",
    )
    .await
}

pub async fn create_module(token: &str, input: CreateModuleInput) -> Result<ContentModuleView> {
    let mut map = serde_json::to_value(input)?
        .as_object()
        .cloned()
        .unwrap_or_default();
    request_typed("content.module.create", token_payload(token, map), "content plugin").await
}

pub async fn update_module(
    token: &str,
    input: UpdateModuleInput,
) -> Result<ContentModuleView> {
    let mut map = serde_json::to_value(input)?
        .as_object()
        .cloned()
        .unwrap_or_default();
    request_typed("content.module.update", token_payload(token, map), "content plugin").await
}

pub async fn delete_module(token: &str, module_id: uuid::Uuid) -> Result<()> {
    request_typed::<serde_json::Value>(
        "content.module.delete",
        token_payload(
            token,
            serde_json::Map::from_iter([("module_id".to_string(), json!(module_id))]),
        ),
        "content plugin",
    )
    .await?;
    Ok(())
}

pub async fn reorder_modules(
    token: &str,
    course_id: uuid::Uuid,
    module_ids: Vec<uuid::Uuid>,
) -> Result<()> {
    let mut map = serde_json::Map::new();
    map.insert("course_id".to_string(), json!(course_id));
    map.insert("module_ids".to_string(), json!(module_ids));
    request_typed::<serde_json::Value>("content.module.reorder", token_payload(token, map), "content plugin").await?;
    Ok(())
}

pub async fn set_module_status(
    token: &str,
    module_id: uuid::Uuid,
    status: &str,
) -> Result<ContentModuleView> {
    request_typed(
        "content.module.set_status",
        token_payload(
            token,
            serde_json::Map::from_iter([
                ("module_id".to_string(), json!(module_id)),
                ("status".to_string(), json!(status)),
            ]),
        ),
        "content plugin",
    )
    .await
}

// ---------------------------------------------------------------------------
// Lessons
// ---------------------------------------------------------------------------

pub async fn list_lessons(
    token: &str,
    module_id: uuid::Uuid,
) -> Result<Vec<ContentLessonView>> {
    request_typed(
        "content.lesson.list",
        token_payload(
            token,
            serde_json::Map::from_iter([("module_id".to_string(), json!(module_id))]),
        ),
        "content plugin",
    )
    .await
}

pub async fn get_lesson(token: &str, lesson_id: uuid::Uuid) -> Result<ContentLessonView> {
    request_typed(
        "content.lesson.get",
        token_payload(
            token,
            serde_json::Map::from_iter([("lesson_id".to_string(), json!(lesson_id))]),
        ),
        "content plugin",
    )
    .await
}

pub async fn create_lesson(token: &str, input: CreateLessonInput) -> Result<ContentLessonView> {
    let mut map = serde_json::to_value(input)?
        .as_object()
        .cloned()
        .unwrap_or_default();
    request_typed("content.lesson.create", token_payload(token, map), "content plugin").await
}

pub async fn update_lesson(
    token: &str,
    input: UpdateLessonInput,
) -> Result<ContentLessonView> {
    let mut map = serde_json::to_value(input)?
        .as_object()
        .cloned()
        .unwrap_or_default();
    request_typed("content.lesson.update", token_payload(token, map), "content plugin").await
}

pub async fn delete_lesson(token: &str, lesson_id: uuid::Uuid) -> Result<()> {
    request_typed::<serde_json::Value>(
        "content.lesson.delete",
        token_payload(
            token,
            serde_json::Map::from_iter([("lesson_id".to_string(), json!(lesson_id))]),
        ),
        "content plugin",
    )
    .await?;
    Ok(())
}

pub async fn reorder_lessons(
    token: &str,
    module_id: uuid::Uuid,
    lesson_ids: Vec<uuid::Uuid>,
) -> Result<()> {
    let mut map = serde_json::Map::new();
    map.insert("module_id".to_string(), json!(module_id));
    map.insert("lesson_ids".to_string(), json!(lesson_ids));
    request_typed::<serde_json::Value>("content.lesson.reorder", token_payload(token, map), "content plugin").await?;
    Ok(())
}

pub async fn set_lesson_status(
    token: &str,
    lesson_id: uuid::Uuid,
    status: &str,
) -> Result<ContentLessonView> {
    request_typed(
        "content.lesson.set_status",
        token_payload(
            token,
            serde_json::Map::from_iter([
                ("lesson_id".to_string(), json!(lesson_id)),
                ("status".to_string(), json!(status)),
            ]),
        ),
        "content plugin",
    )
    .await
}

// ---------------------------------------------------------------------------
// Content Items
// ---------------------------------------------------------------------------

pub async fn list_items(
    token: &str,
    lesson_id: uuid::Uuid,
) -> Result<Vec<ContentItemView>> {
    request_typed(
        "content.item.list",
        token_payload(
            token,
            serde_json::Map::from_iter([("lesson_id".to_string(), json!(lesson_id))]),
        ),
        "content plugin",
    )
    .await
}

pub async fn get_item(token: &str, item_id: uuid::Uuid) -> Result<ContentItemView> {
    request_typed(
        "content.item.get",
        token_payload(
            token,
            serde_json::Map::from_iter([("id".to_string(), json!(item_id))]),
        ),
        "content plugin",
    )
    .await
}

pub async fn create_item(
    token: &str,
    input: CreateContentItemInput,
) -> Result<ContentItemView> {
    let mut map = serde_json::to_value(input)?
        .as_object()
        .cloned()
        .unwrap_or_default();
    request_typed("content.item.create", token_payload(token, map), "content plugin").await
}

pub async fn update_item(
    token: &str,
    input: UpdateContentItemInput,
) -> Result<ContentItemView> {
    let mut map = serde_json::to_value(input)?
        .as_object()
        .cloned()
        .unwrap_or_default();
    request_typed("content.item.update", token_payload(token, map), "content plugin").await
}

pub async fn delete_item(token: &str, item_id: uuid::Uuid) -> Result<()> {
    request_typed::<serde_json::Value>(
        "content.item.delete",
        token_payload(
            token,
            serde_json::Map::from_iter([("id".to_string(), json!(item_id))]),
        ),
        "content plugin",
    )
    .await?;
    Ok(())
}

pub async fn reorder_items(
    token: &str,
    lesson_id: uuid::Uuid,
    item_ids: Vec<uuid::Uuid>,
) -> Result<()> {
    let mut map = serde_json::Map::new();
    map.insert("lesson_id".to_string(), json!(lesson_id));
    map.insert("item_ids".to_string(), json!(item_ids));
    request_typed::<serde_json::Value>("content.item.reorder", token_payload(token, map), "content plugin").await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Files
// ---------------------------------------------------------------------------

pub async fn upload_file(
    token: &str,
    course_id: uuid::Uuid,
    filename: &str,
    mime_type: &str,
    size_bytes: i64,
    content_base64: &str,
) -> Result<ContentFileView> {
    let mut map = serde_json::Map::new();
    map.insert("course_id".to_string(), json!(course_id));
    map.insert("filename".to_string(), json!(filename));
    map.insert("mime_type".to_string(), json!(mime_type));
    map.insert("size_bytes".to_string(), json!(size_bytes));
    map.insert("content_base64".to_string(), json!(content_base64));
    request_typed("content.file.upload", token_payload(token, map), "content plugin").await
}

pub async fn list_files(
    token: &str,
    course_id: uuid::Uuid,
) -> Result<Vec<ContentFileView>> {
    request_typed(
        "content.file.list",
        token_payload(
            token,
            serde_json::Map::from_iter([("course_id".to_string(), json!(course_id))]),
        ),
        "content plugin",
    )
    .await
}

pub async fn get_file(token: &str, file_id: uuid::Uuid) -> Result<ContentFileView> {
    request_typed(
        "content.file.get",
        token_payload(
            token,
            serde_json::Map::from_iter([("id".to_string(), json!(file_id))]),
        ),
        "content plugin",
    )
    .await
}

pub async fn download_file(token: &str, file_id: uuid::Uuid) -> Result<FileDownloadView> {
    request_typed(
        "content.file.download",
        token_payload(
            token,
            serde_json::Map::from_iter([("id".to_string(), json!(file_id))]),
        ),
        "content plugin",
    )
    .await
}

pub async fn delete_file(token: &str, file_id: uuid::Uuid) -> Result<()> {
    request_typed::<serde_json::Value>(
        "content.file.delete",
        token_payload(
            token,
            serde_json::Map::from_iter([("id".to_string(), json!(file_id))]),
        ),
        "content plugin",
    )
    .await?;
    Ok(())
}
