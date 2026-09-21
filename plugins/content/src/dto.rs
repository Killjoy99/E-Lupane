//! Request/response payloads exchanged over the message envelope.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    #[default]
    Text,
    Video,
    Audio,
    Pdf,
    Word,
    Powerpoint,
    Image,
    ExternalLink,
    Embedded,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Video => "video",
            ContentType::Audio => "audio",
            ContentType::Pdf => "pdf",
            ContentType::Word => "word",
            ContentType::Powerpoint => "powerpoint",
            ContentType::Image => "image",
            ContentType::ExternalLink => "external_link",
            ContentType::Embedded => "embedded",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(ContentType::Text),
            "video" => Some(ContentType::Video),
            "audio" => Some(ContentType::Audio),
            "pdf" => Some(ContentType::Pdf),
            "word" => Some(ContentType::Word),
            "powerpoint" => Some(ContentType::Powerpoint),
            "image" => Some(ContentType::Image),
            "external_link" => Some(ContentType::ExternalLink),
            "embedded" => Some(ContentType::Embedded),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishStatus {
    #[default]
    Draft,
    Published,
    Unpublished,
}

impl PublishStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PublishStatus::Draft => "draft",
            PublishStatus::Published => "published",
            PublishStatus::Unpublished => "unpublished",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(PublishStatus::Draft),
            "published" => Some(PublishStatus::Published),
            "unpublished" => Some(PublishStatus::Unpublished),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateModuleRequest {
    pub course_id: Uuid,
    /// Optional subject under the course this module belongs to.
    #[serde(default)]
    pub subject_id: Option<Uuid>,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModuleRequest {
    pub module_id: Uuid,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i32>,
    /// Optional re-scope to a subject (None leaves it unchanged).
    #[serde(default)]
    pub subject_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderModulesRequest {
    pub course_id: Uuid,
    pub module_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct SetModuleStatusRequest {
    pub module_id: Uuid,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLessonRequest {
    pub module_id: Uuid,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLessonRequest {
    pub lesson_id: Uuid,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderLessonsRequest {
    pub module_id: Uuid,
    pub lesson_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct SetLessonStatusRequest {
    pub lesson_id: Uuid,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateContentItemRequest {
    pub lesson_id: Uuid,
    pub content_type: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub file_id: Option<Uuid>,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContentItemRequest {
    pub item_id: Uuid,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub file_id: Option<Uuid>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderContentItemsRequest {
    pub lesson_id: Uuid,
    pub item_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UploadFileRequest {
    pub course_id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_base64: String,
}

/// Upload a document/video and attach it directly to a course (auto-creating
/// a "Course Materials" module + "General" lesson when the course has none).
#[derive(Debug, Deserialize)]
pub struct AddCourseMaterialRequest {
    pub course_id: Uuid,
    /// Optional display title; falls back to the file name.
    #[serde(default)]
    pub title: Option<String>,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_base64: String,
}

/// The result of adding course material: the container the item was placed in,
/// the created content item, and the stored file.
#[derive(Debug, Serialize)]
pub struct CourseMaterialView {
    pub module: ContentModuleView,
    pub lesson: ContentLessonView,
    pub item: ContentItemView,
    pub file: ContentFileView,
}

#[derive(Debug, Deserialize)]
pub struct IdRequest {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CourseIdRequest {
    pub course_id: Uuid,
    /// Optional subject-scope: only modules belonging to this subject.
    #[serde(default)]
    pub subject_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ModuleIdRequest {
    pub module_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct LessonIdRequest {
    pub lesson_id: Uuid,
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ContentModuleView {
    pub id: Uuid,
    pub course_id: Uuid,
    /// The subject (module unit under the course) this module belongs to, if any.
    pub subject_id: Option<Uuid>,
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct ContentFileView {
    pub id: Uuid,
    pub course_id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FileDownloadView {
    pub file_id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub content_base64: String,
}
