//! Types mirrored from the enrollments plugin's views. Compiled on BOTH the
//! client (WASM) and the server so server functions and the dashboard can
//! share them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentView {
    pub id: Uuid,
    pub username: String,
    pub full_name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnrollmentView {
    pub id: Uuid,
    pub course_id: Uuid,
    pub student: StudentView,
    pub state: String,
    pub source: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One entry of an enrollment's audit trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryView {
    pub id: Uuid,
    pub enrollment_id: Uuid,
    pub from_state: Option<String>,
    pub to_state: String,
    pub changed_by: Option<Uuid>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InviteView {
    pub id: Uuid,
    pub course_id: Uuid,
    pub token: String,
    pub created_by: Uuid,
    pub max_uses: Option<i64>,
    pub uses: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

/// Per-student result of a bulk enrollment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkEntryResult {
    pub student: String,
    pub ok: bool,
    pub message: String,
    #[serde(default)]
    pub enrollment: Option<EnrollmentView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkEnrollView {
    pub course_id: Uuid,
    pub enrolled: usize,
    pub failed: usize,
    pub results: Vec<BulkEntryResult>,
}

/// Result of an access check for course content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessView {
    pub course_id: Uuid,
    pub allowed: bool,
    pub state: Option<String>,
    pub manager: bool,
}

// ---------------------------------------------------------------------------
// State helpers (UI) — unit-tested
// ---------------------------------------------------------------------------

pub const ENROLLMENT_STATES: [&str; 6] = [
    "pending",
    "active",
    "suspended",
    "completed",
    "expired",
    "cancelled",
];

/// Human-readable label for an enrollment state.
pub fn state_label(state: &str) -> &'static str {
    match state {
        "pending" => "Pending",
        "active" => "Active",
        "suspended" => "Suspended",
        "completed" => "Completed",
        "expired" => "Expired",
        "cancelled" => "Cancelled",
        _ => "Unknown",
    }
}

/// State-chip tone (matches the dashboard's StatusChip tones).
pub fn state_tone(state: &str) -> &'static str {
    match state {
        "active" => "ok",
        "pending" => "warn",
        "suspended" => "warn",
        "expired" => "muted",
        "cancelled" => "muted",
        "completed" => "info",
        _ => "muted",
    }
}

/// States a manager can move an enrollment into from `state`.
pub fn available_transitions(state: &str) -> &'static [(&'static str, &'static str)] {
    match state {
        "pending" => &[("approve", "Approve"), ("cancel", "Reject")],
        "active" => &[
            ("suspend", "Suspend"),
            ("complete", "Mark completed"),
            ("cancel", "Cancel"),
        ],
        "suspended" => &[("approve", "Reinstate"), ("cancel", "Cancel")],
        "completed" | "expired" | "cancelled" => &[("reenroll", "Re-enroll")],
        _ => &[],
    }
}

/// Whether a student-facing action is available on their own enrollment.
pub fn own_action(state: &str) -> Option<(&'static str, &'static str)> {
    match state {
        "pending" | "active" | "suspended" => Some(("cancel", "Withdraw")),
        "expired" | "cancelled" | "completed" => Some(("reenroll", "Re-enroll")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_tones() {
        assert_eq!(state_label("active"), "Active");
        assert_eq!(state_label("pending"), "Pending");
        assert_eq!(state_label("bogus"), "Unknown");
        assert_eq!(state_tone("active"), "ok");
        assert_eq!(state_tone("pending"), "warn");
        assert_eq!(state_tone("cancelled"), "muted");
    }

    #[test]
    fn manager_transitions() {
        assert_eq!(available_transitions("pending").len(), 2);
        assert_eq!(available_transitions("active").len(), 3);
        assert_eq!(available_transitions("suspended").len(), 2);
        assert_eq!(available_transitions("expired").len(), 1);
        assert_eq!(available_transitions("completed").len(), 1);
        assert_eq!(available_transitions("cancelled").len(), 1);
        assert_eq!(available_transitions("bogus").len(), 0);
    }

    #[test]
    fn student_actions() {
        assert_eq!(own_action("active"), Some(("cancel", "Withdraw")));
        assert_eq!(own_action("pending"), Some(("cancel", "Withdraw")));
        assert_eq!(own_action("expired"), Some(("reenroll", "Re-enroll")));
        assert_eq!(own_action("cancelled"), Some(("reenroll", "Re-enroll")));
        assert!(own_action("bogus").is_none());
    }
}