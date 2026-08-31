//! Request/response payloads exchanged over the message envelope.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// What kind of content a submission may carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionType {
    File,
    Text,
    Link,
    #[default]
    Any,
}

impl SubmissionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubmissionType::File => "file",
            SubmissionType::Text => "text",
            SubmissionType::Link => "link",
            SubmissionType::Any => "any",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "file" => Some(SubmissionType::File),
            "text" => Some(SubmissionType::Text),
            "link" => Some(SubmissionType::Link),
            "any" => Some(SubmissionType::Any),
            _ => None,
        }
    }
}

/// How late submissions are treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LateSubmissionPolicy {
    /// No submissions accepted after the deadline.
    Disallowed,
    /// Late submissions accepted with no penalty.
    Allowed,
    /// Late submissions accepted but penalised by `late_penalty_percent`.
    #[default]
    AllowedWithPenalty,
}

impl LateSubmissionPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            LateSubmissionPolicy::Disallowed => "disallowed",
            LateSubmissionPolicy::Allowed => "allowed",
            LateSubmissionPolicy::AllowedWithPenalty => "allowed_with_penalty",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "disallowed" => Some(LateSubmissionPolicy::Disallowed),
            "allowed" => Some(LateSubmissionPolicy::Allowed),
            "allowed_with_penalty" => Some(LateSubmissionPolicy::AllowedWithPenalty),
            _ => None,
        }
    }
}

fn default_max_marks() -> f64 {
    100.0
}

fn default_allowed_attempts() -> i32 {
    1
}

// ---------------------------------------------------------------------------
// Assignments
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateAssignmentRequest {
    pub title: String,
    #[serde(default)]
    pub instructions: String,
    #[serde(default)]
    pub course_id: Option<Uuid>,
    #[serde(default)]
    pub deadline: Option<DateTime<Utc>>,
    #[serde(default = "default_max_marks")]
    pub max_marks: f64,
    #[serde(default = "default_allowed_attempts")]
    pub allowed_attempts: i32,
    #[serde(default)]
    pub submission_type: SubmissionType,
    #[serde(default)]
    pub late_submission_policy: LateSubmissionPolicy,
    #[serde(default)]
    pub late_penalty_percent: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssignmentRequest {
    pub assignment_id: Uuid,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub deadline: Option<DateTime<Utc>>,
    #[serde(default)]
    pub max_marks: Option<f64>,
    #[serde(default)]
    pub allowed_attempts: Option<i32>,
    #[serde(default)]
    pub submission_type: Option<SubmissionType>,
    #[serde(default)]
    pub late_submission_policy: Option<LateSubmissionPolicy>,
    #[serde(default)]
    pub late_penalty_percent: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct AssignmentIdRequest {
    pub assignment_id: Uuid,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct AssignmentList {
    pub assignments: Vec<AssignmentView>,
}

// ---------------------------------------------------------------------------
// Resources (instructor attachments)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AttachResourceRequest {
    pub assignment_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub mime_type: String,
    /// Raw file bytes, base64-encoded.
    pub content_base64: String,
}

#[derive(Debug, Deserialize)]
pub struct ResourceIdRequest {
    pub resource_id: Uuid,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct DownloadView {
    pub name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
    /// File bytes, base64-encoded.
    pub content_base64: String,
}


// ---------------------------------------------------------------------------
// Submissions
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SubmitRequest {
    pub assignment_id: Uuid,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub files: Vec<FileUpload>,
    #[serde(default)]
    pub links: Vec<LinkUpload>,
}

#[derive(Debug, Deserialize)]
pub struct FileUpload {
    pub name: String,
    #[serde(default)]
    pub mime_type: String,
    /// Raw file bytes, base64-encoded.
    pub content_base64: String,
}

#[derive(Debug, Deserialize)]
pub struct LinkUpload {
    pub url: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmissionStatusRequest {
    pub assignment_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SubmissionFileRequest {
    pub submission_file_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct SubmissionView {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub student_id: Uuid,
    pub attempt: i32,
    pub text: Option<String>,
    pub files: Vec<FileView>,
    pub links: Vec<LinkView>,
    pub submitted_at: DateTime<Utc>,
    pub is_late: bool,
    pub status: String,
    pub graded: bool,
}

#[derive(Debug, Serialize)]
pub struct FileView {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
}

#[derive(Debug, Serialize)]
pub struct LinkView {
    pub id: Uuid,
    pub url: String,
    pub title: String,
}

// ---------------------------------------------------------------------------
// Grading
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct GradeRequest {
    pub submission_id: Uuid,
    pub marks: f64,
    #[serde(default)]
    pub comments: String,
    #[serde(default)]
    pub feedback: String,
}

#[derive(Debug, Deserialize)]
pub struct GradeHistoryRequest {
    pub submission_id: Uuid,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct GradeHistoryView {
    pub entries: Vec<GradeHistoryEntry>,
}

// ---------------------------------------------------------------------------
// Rubrics
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateRubricRequest {
    pub assignment_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// `[{ "name": ..., "max_marks": ..., "description": ... }]`
    #[serde(default)]
    pub criteria: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct RubricIdRequest {
    pub rubric_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct RubricView {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub name: String,
    pub description: String,
    pub criteria: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RubricList {
    pub rubrics: Vec<RubricView>,
}

