//! Row types mapped with `sqlx::FromRow`.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// An enrollment row joined with the student's identity from `users`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Enrollment {
    pub id: Uuid,
    pub course_id: Uuid,
    pub student_id: Uuid,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub state: String,
    pub source: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One enrollment audit-trail entry.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub enrollment_id: Uuid,
    pub from_state: Option<String>,
    pub to_state: String,
    pub changed_by: Option<Uuid>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A course invite link.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Invite {
    pub id: Uuid,
    pub course_id: Uuid,
    pub token: String,
    pub created_by: Uuid,
    pub max_uses: Option<i32>,
    pub uses: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}