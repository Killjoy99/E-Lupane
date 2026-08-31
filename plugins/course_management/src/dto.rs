//! Request/response payloads exchanged over the message envelope.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Course statuses
// ---------------------------------------------------------------------------

/// The lifecycle statuses of a course.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CourseStatus {
    /// Initial state: being authored, not visible to students.
    #[default]
    Draft,
    /// Submitted for review by a platform admin/reviewer.
    PendingReview,
    /// Live and visible to students.
    Published,
    /// Taken down but not deleted (e.g. temporarily).
    Unpublished,
    /// Removed from active use; kept for record/restoration.
    Archived,
}

impl CourseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CourseStatus::Draft => "draft",
            CourseStatus::PendingReview => "pending_review",
            CourseStatus::Published => "published",
            CourseStatus::Unpublished => "unpublished",
            CourseStatus::Archived => "archived",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(CourseStatus::Draft),
            "pending_review" => Some(CourseStatus::PendingReview),
            "published" => Some(CourseStatus::Published),
            "unpublished" => Some(CourseStatus::Unpublished),
            "archived" => Some(CourseStatus::Archived),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub const ALL: [CourseStatus; 5] = [
        CourseStatus::Draft,
        CourseStatus::PendingReview,
        CourseStatus::Published,
        CourseStatus::Unpublished,
        CourseStatus::Archived,
    ];
}

/// Whether a transition from `from` to `to` is valid.
///
/// * Draft      → Pending Review (submit), Archived
/// * Pending    → Published (approved), Unpublished (held/rejected), Draft
/// * Published  → Unpublished, Archived
/// * Unpublished → Published, Archived, Draft
/// * Archived   → Draft (restore), Unpublished
pub fn can_transition(from: CourseStatus, to: CourseStatus) -> bool {
    if from == to {
        return true;
    }
    match from {
        CourseStatus::Draft => matches!(to, CourseStatus::PendingReview | CourseStatus::Archived),
        CourseStatus::PendingReview => matches!(
            to,
            CourseStatus::Published
                | CourseStatus::Unpublished
                | CourseStatus::Draft
                | CourseStatus::Archived
        ),
        CourseStatus::Published => matches!(to, CourseStatus::Unpublished | CourseStatus::Archived),
        CourseStatus::Unpublished => matches!(
            to,
            CourseStatus::Published | CourseStatus::Archived | CourseStatus::Draft
        ),
        CourseStatus::Archived => matches!(to, CourseStatus::Draft | CourseStatus::Unpublished),
    }
}

/// Difficulty levels for a course.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CourseDifficulty {
    #[default]
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

impl CourseDifficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            CourseDifficulty::Beginner => "beginner",
            CourseDifficulty::Intermediate => "intermediate",
            CourseDifficulty::Advanced => "advanced",
            CourseDifficulty::Expert => "expert",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "beginner" => Some(CourseDifficulty::Beginner),
            "intermediate" => Some(CourseDifficulty::Intermediate),
            "advanced" => Some(CourseDifficulty::Advanced),
            "expert" => Some(CourseDifficulty::Expert),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

fn default_language() -> String {
    "en".to_string()
}

#[derive(Debug, Deserialize)]
pub struct CreateCourseRequest {
    pub code: String,
    pub title: String,
    #[serde(default)]
    pub short_description: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub thumbnail: String,
    #[serde(default)]
    pub banner: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub level: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub difficulty: String,
    #[serde(default)]
    pub duration: String,
    #[serde(default)]
    pub learning_objectives: Vec<String>,
    #[serde(default)]
    pub target_audience: Vec<String>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}


#[derive(Debug, Deserialize)]
pub struct UpdateCourseRequest {
    pub course_id: Uuid,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub short_description: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub banner: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub difficulty: Option<String>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub learning_objectives: Option<Vec<String>>,
    #[serde(default)]
    pub target_audience: Option<Vec<String>>,
    #[serde(default)]
    pub prerequisites: Option<Vec<String>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CourseIdRequest {
    pub course_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SetStatusRequest {
    pub course_id: Uuid,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct InstructorRequest {
    pub course_id: Uuid,
    pub user_id: Uuid,
}

/// Optional filters for `course.list`.
#[derive(Debug, Deserialize, Default)]
pub struct ListCoursesRequest {
    /// Only courses in this status (`draft`, `published`, ...).
    #[serde(default)]
    pub status: Option<String>,
    /// Only courses the caller can manage (instructors). Ignored for students.
    #[serde(default)]
    pub mine: bool,
    /// Free-text search over code / title / subject / tags.
    #[serde(default)]
    pub search: Option<String>,
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct CourseView {
    pub id: Uuid,
    pub code: String,
    pub title: String,
    pub short_description: String,
    pub description: String,
    pub thumbnail: String,
    pub banner: String,
    pub category: String,
    pub subject: String,
    pub level: String,
    pub language: String,
    pub difficulty: String,
    pub duration: String,
    pub learning_objectives: Vec<String>,
    pub target_audience: Vec<String>,
    pub prerequisites: Vec<String>,
    pub tags: Vec<String>,
    pub status: String,
    pub owner_id: Uuid,
    pub instructors: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CourseList {
    pub courses: Vec<CourseView>,
}
