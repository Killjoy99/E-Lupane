//! Types mirrored from the course_management plugin's views. Compiled on BOTH
//! the client (WASM) and the server so server functions and the dashboard can
//! share them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Payload for creating/updating a course (client-friendly subset).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CourseInput {
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
}

/// The five lifecycle statuses.
pub const COURSE_STATUSES: [&str; 5] = [
    "draft",
    "pending_review",
    "published",
    "unpublished",
    "archived",
];

/// Human-readable label for a course status.
pub fn course_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Draft",
        "pending_review" => "Pending Review",
        "published" => "Published",
        "unpublished" => "Unpublished",
        "archived" => "Archived",
        _ => "Unknown",
    }
}

/// Status-chip tone (matches the dashboard's StatusChip tones).
pub fn course_status_tone(status: &str) -> &'static str {
    match status {
        "published" => "ok",
        "pending_review" => "warn",
        "unpublished" => "muted",
        "archived" => "muted",
        _ => "muted",
    }
}

/// Whether an instructor may publish a course directly from the given status.
pub fn can_publish_from(status: &str) -> bool {
    matches!(status, "draft" | "pending_review" | "unpublished")
}

/// Whether the caller may preview a course (managers only; admins are managers).
pub fn can_manage_course(course: &CourseView, user_id: Uuid, is_admin: bool) -> bool {
    is_admin || course.owner_id == user_id || course.instructors.contains(&user_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_labels() {
        assert_eq!(course_status_label("published"), "Published");
        assert_eq!(course_status_label("pending_review"), "Pending Review");
        assert_eq!(course_status_label("archived"), "Archived");
        assert_eq!(course_status_label("bogus"), "Unknown");
    }

    #[test]
    fn status_tones() {
        assert_eq!(course_status_tone("published"), "ok");
        assert_eq!(course_status_tone("pending_review"), "warn");
        assert_eq!(course_status_tone("draft"), "muted");
    }

    #[test]
    fn publishability() {
        assert!(can_publish_from("draft"));
        assert!(can_publish_from("pending_review"));
        assert!(can_publish_from("unpublished"));
        assert!(!can_publish_from("archived"));
        assert!(!can_publish_from("published"));
    }

    #[test]
    fn course_management_rights() {
        let owner = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let co = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        let other = Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap();
        let course = CourseView {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
            code: "CS101".into(),
            title: "Intro".into(),
            short_description: String::new(),
            description: String::new(),
            thumbnail: String::new(),
            banner: String::new(),
            category: String::new(),
            subject: String::new(),
            level: String::new(),
            language: "en".into(),
            difficulty: "beginner".into(),
            duration: String::new(),
            learning_objectives: Vec::new(),
            target_audience: Vec::new(),
            prerequisites: Vec::new(),
            tags: vec!["cs".into()],
            status: "draft".into(),
            owner_id: owner,
            instructors: vec![co],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(can_manage_course(&course, owner, false));
        assert!(can_manage_course(&course, co, false));
        assert!(!can_manage_course(&course, other, false));
        assert!(can_manage_course(&course, other, true));
    }
}
