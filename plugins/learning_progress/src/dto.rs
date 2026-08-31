use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentCourseRequest {
    pub student_id: Uuid,
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseIdRequest {
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonIdRequest {
    pub lesson_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgressRequest {
    pub lesson_id: Uuid,
    pub progress_pct: Option<f64>,
    pub video_timestamp: Option<f64>,
    pub event_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkCompleteRequest {
    pub lesson_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeLearningRequest {
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningProgressResponse {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseProgressResponse {
    pub course_id: Uuid,
    pub student_id: Uuid,
    pub overall_progress_pct: f64,
    pub modules_completed: i64,
    pub lessons_completed: i64,
    pub total_lessons: i64,
    pub last_activity_at: DateTime<Utc>,
    pub last_lesson_id: Option<Uuid>,
}
