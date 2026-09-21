//! Database row models.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct ContentModule {
    pub id: Uuid,
    pub course_id: Uuid,
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

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct ContentLesson {
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

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct ContentItem {
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

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct ContentFile {
    pub id: Uuid,
    pub course_id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}
