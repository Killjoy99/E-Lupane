//! SQLx data access for enrollments, history and invites.
//!
//! All filters are bound parameters; `AssertSqlSafe` is only used for the
//! constant column lists / table names, matching the other plugins' style.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{AssertSqlSafe, PgPool};
use uuid::Uuid;

use crate::models::{Enrollment, HistoryEntry, Invite};

pub const ENROLLMENT_COLUMNS: &str = "e.id, e.course_id, e.student_id, u.username, \
    u.full_name, u.email, e.state, e.source, e.expires_at, e.created_at, e.updated_at";

const ENROLLMENT_FROM: &str =
    "FROM course_enrollments e JOIN users u ON u.id = e.student_id";

pub async fn find_enrollment_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Enrollment>> {
    Ok(sqlx::query_as::<_, Enrollment>(AssertSqlSafe(format!(
        "SELECT {ENROLLMENT_COLUMNS} {ENROLLMENT_FROM} WHERE e.id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_enrollment(
    pool: &PgPool,
    course_id: Uuid,
    student_id: Uuid,
) -> Result<Option<Enrollment>> {
    Ok(sqlx::query_as::<_, Enrollment>(AssertSqlSafe(format!(
        "SELECT {ENROLLMENT_COLUMNS} {ENROLLMENT_FROM} \
         WHERE e.course_id = $1 AND e.student_id = $2"
    )))
    .bind(course_id)
    .bind(student_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_course_enrollments(
    pool: &PgPool,
    course_id: Uuid,
    state: Option<&str>,
) -> Result<Vec<Enrollment>> {
    Ok(sqlx::query_as::<_, Enrollment>(AssertSqlSafe(format!(
        "SELECT {ENROLLMENT_COLUMNS} {ENROLLMENT_FROM} \
         WHERE e.course_id = $1 AND ($2::text IS NULL OR e.state = $2) \
         ORDER BY e.created_at DESC"
    )))
    .bind(course_id)
    .bind(state)
    .fetch_all(pool)
    .await?)
}

pub async fn list_student_enrollments(
    pool: &PgPool,
    student_id: Uuid,
) -> Result<Vec<Enrollment>> {
    Ok(sqlx::query_as::<_, Enrollment>(AssertSqlSafe(format!(
        "SELECT {ENROLLMENT_COLUMNS} {ENROLLMENT_FROM} \
         WHERE e.student_id = $1 ORDER BY e.created_at DESC"
    )))
    .bind(student_id)
    .fetch_all(pool)
    .await?)
}

/// Count of active/pending enrollments per course — used by the UI roster.
pub async fn count_enrollments(pool: &PgPool, course_id: Uuid, state: &str) -> Result<i64> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM course_enrollments WHERE course_id = $1 AND state = $2",
    )
    .bind(course_id)
    .bind(state)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Create or reset an enrollment (re-enrollment reuses the row).
#[allow(clippy::too_many_arguments)]
pub async fn upsert_enrollment(
    pool: &PgPool,
    course_id: Uuid,
    student_id: Uuid,
    state: &str,
    source: &str,
    enrolled_by: Option<Uuid>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<Uuid> {
    let id: Uuid = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO course_enrollments \
             (course_id, student_id, state, source, enrolled_by, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (course_id, student_id) DO UPDATE SET \
             state = EXCLUDED.state, source = EXCLUDED.source, \
             enrolled_by = EXCLUDED.enrolled_by, expires_at = EXCLUDED.expires_at, \
             approved_by = NULL, approved_at = NULL, completed_at = NULL, \
             suspended_at = NULL, cancelled_at = NULL, updated_at = now() \
         RETURNING id",
    )
    .bind(course_id)
    .bind(student_id)
    .bind(state)
    .bind(source)
    .bind(enrolled_by)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Move an enrollment to `state`, stamping the matching lifecycle timestamp.
#[allow(clippy::too_many_arguments)]
pub async fn update_enrollment_state(
    pool: &PgPool,
    id: Uuid,
    state: &str,
    expires_at: Option<DateTime<Utc>>,
    approved_by: Option<Uuid>,
) -> Result<Uuid> {
    let (approved_by, completed_at, suspended_at, cancelled_at) = match state {
        "active" => (approved_by, None, None, None),
        "completed" => (None, Some(Utc::now()), None, None),
        "suspended" => (None, None, Some(Utc::now()), None),
        "cancelled" => (None, None, None, Some(Utc::now())),
        _ => (None, None, None, None),
    };
    let returned_id: Uuid = sqlx::query_scalar::<_, Uuid>(
        "UPDATE course_enrollments SET \
             state = $2, expires_at = $3, approved_by = $4, approved_at = \
             CASE WHEN $4::uuid IS NOT NULL THEN now() ELSE approved_at END, \
             completed_at = $5, suspended_at = $6, cancelled_at = $7, \
             updated_at = now() \
         WHERE id = $1 \
         RETURNING id",
    )
    .bind(id)
    .bind(state)
    .bind(expires_at)
    .bind(approved_by)
    .bind(completed_at)
    .bind(suspended_at)
    .bind(cancelled_at)
    .fetch_one(pool)
    .await?;
    Ok(returned_id)
}

/// Expire every active enrollment whose deadline has passed (sweep).
pub async fn expire_due_enrollments(pool: &PgPool) -> Result<Vec<Uuid>> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "UPDATE course_enrollments SET state = 'expired', updated_at = now() \
         WHERE state = 'active' AND expires_at IS NOT NULL AND expires_at <= now() \
         RETURNING id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

// ---------------------------------------------------------------------------
// Enrollment rows — insert / fetch / full state update
// ---------------------------------------------------------------------------

pub async fn insert_enrollment(
    pool: &PgPool,
    course_id: Uuid,
    student_id: Uuid,
    state: &str,
    source: &str,
    enrolled_by: Option<Uuid>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<Uuid> {
    let id: Uuid = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO course_enrollments \
             (course_id, student_id, state, source, enrolled_by, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id",
    )
    .bind(course_id)
    .bind(student_id)
    .bind(state)
    .bind(source)
    .bind(enrolled_by)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn get_enrollment(pool: &PgPool, id: Uuid) -> Result<Option<Enrollment>> {
    find_enrollment_by_id(pool, id).await
}

/// Move an enrollment to an explicit state, stamping each lifecycle timestamp
/// from the caller (used by the generic state machine in `services`).
#[allow(clippy::too_many_arguments)]
pub async fn update_enrollment_state_full(
    pool: &PgPool,
    id: Uuid,
    state: &str,
    approved_by: Option<Uuid>,
    approved_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    suspended_at: Option<DateTime<Utc>>,
    cancelled_at: Option<DateTime<Utc>>,
) -> Result<Uuid> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "UPDATE course_enrollments SET \
             state = $2, \
             approved_by = COALESCE($3, approved_by), \
             approved_at = COALESCE($4, approved_at), \
             completed_at = COALESCE($5, completed_at), \
             suspended_at = COALESCE($6, suspended_at), \
             cancelled_at = COALESCE($7, cancelled_at), \
             updated_at = now() \
         WHERE id = $1 \
         RETURNING id",
    )
    .bind(id)
    .bind(state)
    .bind(approved_by)
    .bind(approved_at)
    .bind(completed_at)
    .bind(suspended_at)
    .bind(cancelled_at)
    .fetch_one(pool)
    .await?)
}

/// All `active` enrollments whose deadline has passed (for the sweep).
pub async fn list_expired_active(pool: &PgPool, now: DateTime<Utc>) -> Result<Vec<Enrollment>> {
    Ok(sqlx::query_as::<_, Enrollment>(AssertSqlSafe(format!(
        "SELECT {ENROLLMENT_COLUMNS} {ENROLLMENT_FROM} \
         WHERE e.state = 'active' AND e.expires_at IS NOT NULL AND e.expires_at <= $1"
    )))
    .bind(now)
    .fetch_all(pool)
    .await?)
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

pub async fn insert_history(
    pool: &PgPool,
    enrollment_id: Uuid,
    from_state: Option<&str>,
    to_state: &str,
    changed_by: Option<Uuid>,
    reason: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO enrollment_history \
             (enrollment_id, from_state, to_state, changed_by, reason) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(enrollment_id)
    .bind(from_state)
    .bind(to_state)
    .bind(changed_by)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_history(pool: &PgPool, enrollment_id: Uuid) -> Result<Vec<HistoryEntry>> {
    Ok(sqlx::query_as::<_, HistoryEntry>(
        "SELECT id, enrollment_id, from_state, to_state, changed_by, reason, created_at \
         FROM enrollment_history WHERE enrollment_id = $1 ORDER BY created_at DESC",
    )
    .bind(enrollment_id)
    .fetch_all(pool)
    .await?)
}

// ---------------------------------------------------------------------------
// Invites
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub async fn insert_invite(
    pool: &PgPool,
    course_id: Uuid,
    token: &str,
    created_by: Uuid,
    max_uses: Option<i32>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<Invite> {
    Ok(sqlx::query_as::<_, Invite>(
        "INSERT INTO course_invites (course_id, token, created_by, max_uses, expires_at) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, course_id, token, created_by, max_uses, uses, expires_at, active, created_at",
    )
    .bind(course_id)
    .bind(token)
    .bind(created_by)
    .bind(max_uses)
    .bind(expires_at)
    .fetch_one(pool)
    .await?)
}

pub async fn find_invite_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Invite>> {
    Ok(sqlx::query_as::<_, Invite>(
        "SELECT id, course_id, token, created_by, max_uses, uses, expires_at, active, created_at \
         FROM course_invites WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_invite_by_token(pool: &PgPool, token: &str) -> Result<Option<Invite>> {
    Ok(sqlx::query_as::<_, Invite>(
        "SELECT id, course_id, token, created_by, max_uses, uses, expires_at, active, created_at \
         FROM course_invites WHERE token = $1",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_invites(pool: &PgPool, course_id: Uuid) -> Result<Vec<Invite>> {
    Ok(sqlx::query_as::<_, Invite>(
        "SELECT id, course_id, token, created_by, max_uses, uses, expires_at, active, created_at \
         FROM course_invites WHERE course_id = $1 ORDER BY created_at DESC",
    )
    .bind(course_id)
    .fetch_all(pool)
    .await?)
}

/// Mark an invite as used (+1). Returns the number of rows updated.
pub async fn consume_invite(pool: &PgPool, id: Uuid) -> Result<u64> {
    let result =
        sqlx::query("UPDATE course_invites SET uses = uses + 1 WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
}

/// Deactivate an invite (revoke the course link).
pub async fn revoke_invite(pool: &PgPool, id: Uuid) -> Result<Invite> {
    Ok(sqlx::query_as::<_, Invite>(
        "UPDATE course_invites SET active = FALSE \
         WHERE id = $1 \
         RETURNING id, course_id, token, created_by, max_uses, uses, expires_at, active, created_at",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

// ---------------------------------------------------------------------------
// User lookup (shared `users` table — same as the auth plugin's)
// ---------------------------------------------------------------------------

/// Resolve a username or email to a user identity.
pub async fn find_user_by_identifier(
    pool: &PgPool,
    identifier: &str,
) -> Result<Option<(Uuid, String, String, String)>> {
    let row = sqlx::query_as::<_, (Uuid, String, String, String)>(
        "SELECT id, username, full_name, email FROM users \
         WHERE (lower(username) = lower($1) OR lower(email) = lower($1)) AND active",
    )
    .bind(identifier.trim())
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Resolve a student's username/email, failing with a friendly error.
pub async fn require_user(
    pool: &PgPool,
    identifier: &str,
) -> Result<(Uuid, String, String, String)> {
    find_user_by_identifier(pool, identifier)
        .await?
        .ok_or_else(|| anyhow::anyhow!("no active user found for '{identifier}'"))
}