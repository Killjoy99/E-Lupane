//! Types mirrored from the assignments plugin's views. Compiled on BOTH the
//! client (WASM) and the server so server functions and the dashboard can share
//! them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignmentView {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceView {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmissionView {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub student_id: Uuid,
    pub attempt: i32,
    pub text: Option<String>,
    pub files: Vec<SubmissionFileView>,
    pub links: Vec<SubmissionLinkView>,
    pub submitted_at: DateTime<Utc>,
    pub is_late: bool,
    pub status: String,
    pub graded: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmissionFileView {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmissionLinkView {
    pub id: Uuid,
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradeView {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradeHistoryView {
    pub entries: Vec<GradeHistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadView {
    pub name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
    /// File bytes, base64-encoded.
    pub content_base64: String,
}

/// Payload for creating/updating an assignment (client-friendly subset).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignmentInput {
    pub title: String,
    pub instructions: String,
    pub deadline: Option<DateTime<Utc>>,
    pub max_marks: f64,
    pub allowed_attempts: i32,
    pub submission_type: String,
    pub late_submission_policy: String,
    pub late_penalty_percent: f64,
    /// Optional link to a course managed by the instructor.
    #[serde(default)]
    pub course_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileUploadInput {
    pub name: String,
    pub mime_type: String,
    /// Raw bytes, base64-encoded.
    pub content_base64: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkInput {
    pub url: String,
    pub title: String,
}
