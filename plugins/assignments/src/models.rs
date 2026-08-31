//! Database row models.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Assignment {
    pub id: Uuid,
    pub title: String,
    pub instructions: String,
    pub course_id: Option<Uuid>,
    pub created_by: Uuid,
    pub deadline: Option<DateTime<Utc>>,
    pub max_marks: f64,
    pub allowed_attempts: i32,
    pub submission_type: String,
    pub late_submission_policy: String,
    pub late_penalty_percent: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AssignmentResource {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub name: String,
    pub stored_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Submission {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub student_id: Uuid,
    pub attempt: i32,
    pub text_content: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub is_late: bool,
    pub status: String,
    pub graded: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct SubmissionFile {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub name: String,
    pub stored_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SubmissionLink {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub url: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Grade {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub assignment_id: Uuid,
    pub student_id: Uuid,
    pub grader_id: Uuid,
    pub marks: f64,
    pub raw_marks: f64,
    pub comments: String,
    pub feedback: String,
    pub graded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct GradeHistoryEntry {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub grader_id: Uuid,
    pub marks: f64,
    pub raw_marks: f64,
    pub comments: String,
    pub feedback: String,
    pub graded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Rubric {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub name: String,
    pub description: String,
    pub criteria: Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}
