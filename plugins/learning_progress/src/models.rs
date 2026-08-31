use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LearningProgress {
    pub id: Uuid,
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub module_id: Option<Uuid>,
    pub lesson_id: Option<Uuid>,
    pub content_item_id: Option<Uuid>,
    pub status: String,
    pub progress_pct: f64,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_activity_at: DateTime<Utc>,
    pub last_lesson_id: Option<Uuid>,
    pub video_timestamp: f64,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProgressEvent {
    pub id: Uuid,
    pub progress_id: Uuid,
    pub event_type: String,
    pub data: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
}
