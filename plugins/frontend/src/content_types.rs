//! Types mirrored from the content plugin's views. Compiled on BOTH
//! the client (WASM) and the server so server functions and the dashboard can
//! share them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentModuleView {
    pub id: Uuid,
    pub course_id: Uuid,
    pub title: String,
    pub description: String,
    pub sort_order: i32,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentLessonView {
    pub id: Uuid,
    pub module_id: Uuid,
    pub title: String,
    pub description: String,
    pub sort_order: i32,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentItemView {
    pub id: Uuid,
    pub lesson_id: Uuid,
    pub content_type: String,
    pub title: String,
    pub body: String,
    pub url: String,
    pub file_id: Option<Uuid>,
    pub mime_type: String,
    pub file_size_bytes: Option<i64>,
    pub sort_order: i32,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentFileView {
    pub id: Uuid,
    pub course_id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileDownloadView {
    pub file_id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub content_base64: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateModuleInput {
    pub course_id: Uuid,
    pub title: String,
    pub description: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateModuleInput {
    pub module_id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateLessonInput {
    pub module_id: Uuid,
    pub title: String,
    pub description: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateLessonInput {
    pub lesson_id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateContentItemInput {
    pub lesson_id: Uuid,
    pub content_type: String,
    pub title: String,
    pub body: String,
    pub url: String,
    pub file_id: Option<Uuid>,
    pub mime_type: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateContentItemInput {
    pub item_id: Uuid,
    pub content_type: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub url: Option<String>,
    pub file_id: Option<Uuid>,
    pub mime_type: Option<String>,
    pub sort_order: Option<i32>,
}

pub const CONTENT_TYPES: [&str; 9] = [
    "text", "video", "audio", "pdf", "word", "powerpoint", "image", "external_link", "embedded",
];

pub const CONTENT_STATUSES: [&str; 3] = ["draft", "published", "unpublished"];
