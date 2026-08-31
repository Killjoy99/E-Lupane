//! Request/response payloads exchanged over the message envelope.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enrollment states
// ---------------------------------------------------------------------------

/// The lifecycle states of an enrollment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentState {
    /// Awaiting approval from an instructor/admin.
    #[default]
    Pending,
    /// Fully enrolled: can access course content, submit assignments, attend.
    Active,
    /// Temporarily blocked from accessing course content.
    Suspended,
    /// Finished the course.
    Completed,
    /// Enrollment deadline passed (swept from `active` past `expires_at`).
    Expired,
    /// Withdrawn (by the student) or rejected (by a manager).
    Cancelled,
}

impl EnrollmentState {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnrollmentState::Pending => "pending",
            EnrollmentState::Active => "active",
            EnrollmentState::Suspended => "suspended",
            EnrollmentState::Completed => "completed",
            EnrollmentState::Expired => "expired",
            EnrollmentState::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(EnrollmentState::Pending),
            "active" => Some(EnrollmentState::Active),
            "suspended" => Some(EnrollmentState::Suspended),
            "completed" => Some(EnrollmentState::Completed),
            "expired" => Some(EnrollmentState::Expired),
            "cancelled" => Some(EnrollmentState::Cancelled),
            _ => None,
        }
    }

    pub const ALL: [EnrollmentState; 6] = [
        EnrollmentState::Pending,
        EnrollmentState::Active,
        EnrollmentState::Suspended,
        EnrollmentState::Completed,
        EnrollmentState::Expired,
        EnrollmentState::Cancelled,
    ];

    /// Whether the state grants access to course content.
    pub fn grants_access(&self) -> bool {
        matches!(self, EnrollmentState::Active)
    }

    /// Whether the enrollment can be restarted (re-enrollment).
    pub fn can_re_enroll(&self) -> bool {
        matches!(
            self,
            EnrollmentState::Cancelled | EnrollmentState::Expired | EnrollmentState::Completed
        )
    }
}

/// Whether a transition from `from` to `to` is valid.
///
/// * Pending   → Active (approve), Cancelled (reject), Expired
/// * Active    → Suspended, Completed, Cancelled, Expired
/// * Suspended → Active (reinstate), Cancelled, Expired
/// * Completed → Active, Pending (re-enroll)
/// * Expired   → Active, Pending (re-enroll)
/// * Cancelled → Pending, Active (re-enroll)
pub fn can_transition(from: EnrollmentState, to: EnrollmentState) -> bool {
    if from == to {
        return true;
    }
    match from {
        EnrollmentState::Pending => {
            matches!(to, EnrollmentState::Active | EnrollmentState::Cancelled | EnrollmentState::Expired)
        }
        EnrollmentState::Active => matches!(
            to,
            EnrollmentState::Suspended
                | EnrollmentState::Completed
                | EnrollmentState::Cancelled
                | EnrollmentState::Expired
        ),
        EnrollmentState::Suspended => matches!(
            to,
            EnrollmentState::Active | EnrollmentState::Cancelled | EnrollmentState::Expired
        ),
        EnrollmentState::Completed => matches!(to, EnrollmentState::Active | EnrollmentState::Pending),
        EnrollmentState::Expired => matches!(to, EnrollmentState::Active | EnrollmentState::Pending),
        EnrollmentState::Cancelled => matches!(to, EnrollmentState::Pending | EnrollmentState::Active),
    }
}

/// Where an enrollment came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentSource {
    /// The student enrolled themselves (lands in `pending`).
    #[default]
    Self_,
    /// Enrolled manually by the course instructor.
    Instructor,
    /// Enrolled by a platform administrator.
    Admin,
    /// Enrolled as part of a bulk operation.
    Bulk,
    /// Joined via a course invite link.
    Invite,
}

impl EnrollmentSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnrollmentSource::Self_ => "self",
            EnrollmentSource::Instructor => "instructor",
            EnrollmentSource::Admin => "admin",
            EnrollmentSource::Bulk => "bulk",
            EnrollmentSource::Invite => "invite",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "self" => Some(EnrollmentSource::Self_),
            "instructor" => Some(EnrollmentSource::Instructor),
            "admin" => Some(EnrollmentSource::Admin),
            "bulk" => Some(EnrollmentSource::Bulk),
            "invite" => Some(EnrollmentSource::Invite),
            _ => None,
        }
    }
}
// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct CourseIdRequest {
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnrollmentIdRequest {
    pub enrollment_id: Uuid,
}

/// Self-enrollment or manual single enrollment.
#[derive(Debug, Clone, Deserialize)]
pub struct EnrollRequest {
    pub course_id: Uuid,
    /// Only for instructor/admin enrollment: the student's username or email.
    #[serde(default)]
    pub student: Option<String>,
    /// Optional enrollment deadline (RFC 3339).
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Bulk enrollment: enroll many students at once.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkEnrollRequest {
    pub course_id: Uuid,
    /// Usernames or emails, one per entry.
    pub students: Vec<String>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

/// A state change (approve / suspend / cancel / ... share one shape).
#[derive(Debug, Clone, Deserialize)]
pub struct ChangeStateRequest {
    pub enrollment_id: Uuid,
    #[serde(default)]
    pub reason: Option<String>,
}

/// List enrollments for a course (managers) or the caller's own (students).
#[derive(Debug, Clone, Deserialize)]
pub struct ListEnrollmentsRequest {
    pub course_id: Uuid,
    #[serde(default)]
    pub state: Option<String>,
}

/// Create an invite link for a course.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateInviteRequest {
    pub course_id: Uuid,
    /// How long the invite stays valid.
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
    /// Maximum number of students that may use it.
    #[serde(default)]
    pub max_uses: Option<i64>,
}

/// Join a course via an invite token (the "course link" flow).
///
/// Named `invite_token` so it doesn't clash with the caller's JWT `token`
/// field in the same payload.
#[derive(Debug, Clone, Deserialize)]
pub struct JoinRequest {
    pub invite_token: String,
}

/// Generic state change request (set_state op).
#[derive(Debug, Clone, Deserialize)]
pub struct SetStateRequest {
    pub enrollment_id: Uuid,
    pub state: String,
    #[serde(default)]
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

/// A student identity resolved from the shared `users` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentView {
    pub id: Uuid,
    pub username: String,
    pub full_name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryView {
    pub id: Uuid,
    pub enrollment_id: Uuid,
    pub from_state: Option<String>,
    pub to_state: String,
    pub changed_by: Option<Uuid>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkEntryResult {
    pub student: String,
    pub ok: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment: Option<EnrollmentView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkEnrollView {
    pub course_id: Uuid,
    pub enrolled: usize,
    pub failed: usize,
    pub results: Vec<BulkEntryResult>,
}

/// Result of an access check for course content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessView {
    pub course_id: Uuid,
    pub allowed: bool,
    /// The caller's enrollment state, if enrolled.
    pub state: Option<String>,
    /// True when the caller manages the course (instructor/admin).
    pub manager: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollmentsView {
    pub enrollments: Vec<EnrollmentView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntriesView {
    pub history: Vec<HistoryView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitesView {
    pub invites: Vec<InviteView>,
}

// ---------------------------------------------------------------------------
// Pure validation helpers (unit-tested)
// ---------------------------------------------------------------------------

/// Validate an invite token's shape: url-safe, non-trivial.
pub fn valid_token_shape(token: &str) -> bool {
    let t = token.trim();
    t.len() >= 16
        && t.len() <= 128
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Generate a url-safe invite token from two random UUIDs (64 hex chars).
pub fn generate_invite_token() -> String {
    let a = Uuid::new_v4().simple().to_string();
    let b = Uuid::new_v4().simple().to_string();
    format!("{a}{b}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trips() {
        for s in EnrollmentState::ALL {
            assert_eq!(EnrollmentState::from_str(s.as_str()), Some(s));
        }
        assert_eq!(EnrollmentState::from_str("bogus"), None);
    }

    #[test]
    fn approval_flow() {
        assert!(can_transition(EnrollmentState::Pending, EnrollmentState::Active));
        assert!(can_transition(EnrollmentState::Pending, EnrollmentState::Cancelled));
        assert!(!can_transition(EnrollmentState::Pending, EnrollmentState::Suspended));
        assert!(!can_transition(EnrollmentState::Pending, EnrollmentState::Completed));
    }

    #[test]
    fn active_lifecycle() {
        assert!(can_transition(EnrollmentState::Active, EnrollmentState::Suspended));
        assert!(can_transition(EnrollmentState::Active, EnrollmentState::Completed));
        assert!(can_transition(EnrollmentState::Active, EnrollmentState::Cancelled));
        assert!(can_transition(EnrollmentState::Active, EnrollmentState::Expired));
        assert!(!can_transition(EnrollmentState::Active, EnrollmentState::Pending));
    }

    #[test]
    fn suspension_cycle() {
        assert!(can_transition(EnrollmentState::Suspended, EnrollmentState::Active));
        assert!(can_transition(EnrollmentState::Suspended, EnrollmentState::Cancelled));
        assert!(!can_transition(EnrollmentState::Suspended, EnrollmentState::Completed));
    }

    #[test]
    fn re_enrollment_flows() {
        assert!(can_transition(EnrollmentState::Cancelled, EnrollmentState::Active));
        assert!(can_transition(EnrollmentState::Cancelled, EnrollmentState::Pending));
        assert!(can_transition(EnrollmentState::Expired, EnrollmentState::Active));
        assert!(can_transition(EnrollmentState::Expired, EnrollmentState::Pending));
        assert!(can_transition(EnrollmentState::Completed, EnrollmentState::Active));
        assert!(!can_transition(EnrollmentState::Completed, EnrollmentState::Suspended));
        assert!(EnrollmentState::Cancelled.can_re_enroll());
        assert!(EnrollmentState::Expired.can_re_enroll());
        assert!(EnrollmentState::Completed.can_re_enroll());
        assert!(!EnrollmentState::Active.can_re_enroll());
        assert!(!EnrollmentState::Pending.can_re_enroll());
    }

    #[test]
    fn only_active_grants_content_access() {
        for s in EnrollmentState::ALL {
            assert_eq!(s.grants_access(), s == EnrollmentState::Active);
        }
    }

    #[test]
    fn source_round_trips() {
        for src in ["self", "instructor", "admin", "bulk", "invite"] {
            let parsed = EnrollmentSource::from_str(src).unwrap();
            assert_eq!(parsed.as_str(), src);
        }
        assert_eq!(EnrollmentSource::from_str("nope"), None);
    }

    #[test]
    fn invite_token_shape() {
        let token = generate_invite_token();
        assert_eq!(token.len(), 64);
        assert!(valid_token_shape(&token));
        assert_ne!(generate_invite_token(), generate_invite_token());
        assert!(!valid_token_shape("short"));
        assert!(!valid_token_shape("has spaces and is long enough!!"));
        assert!(!valid_token_shape(""));
    }
}