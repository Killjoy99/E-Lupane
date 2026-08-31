//! SQLx data access for sessions, attendance, chat, resources and recordings.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{AssertSqlSafe, PgPool};
use uuid::Uuid;

use crate::models::{Attendance, ChatMessage, LiveSession, Recording, SessionResource};

const SESSION_COLUMNS: &str = "id, course_id, title, description, host_id, provider, \
    external_url, scheduled_start, scheduled_end, started_at, ended_at, status, \
    created_at, updated_at";

pub async fn insert_session(
    pool: &PgPool,
    course_id: Uuid,
    title: &str,
    description: &str,
    host_id: Uuid,
    provider: &str,
    external_url: Option<&str>,
    scheduled_start: DateTime<Utc>,
    scheduled_end: DateTime<Utc>,
) -> Result<LiveSession> {
    Ok(sqlx::query_as::<_, LiveSession>(AssertSqlSafe(format!(
        "INSERT INTO live_sessions \
            (course_id, title, description, host_id, provider, external_url, \
             scheduled_start, scheduled_end) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING {SESSION_COLUMNS}"
    )))
    .bind(course_id)
    .bind(title)
    .bind(description)
    .bind(host_id)
    .bind(provider)
    .bind(external_url)
    .bind(scheduled_start)
    .bind(scheduled_end)
    .fetch_one(pool)
    .await?)
}

pub async fn find_session(pool: &PgPool, id: Uuid) -> Result<Option<LiveSession>> {
    Ok(sqlx::query_as::<_, LiveSession>(AssertSqlSafe(format!(
        "SELECT {SESSION_COLUMNS} FROM live_sessions WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_course_sessions(pool: &PgPool, course_id: Uuid) -> Result<Vec<LiveSession>> {
    Ok(sqlx::query_as::<_, LiveSession>(AssertSqlSafe(format!(
        "SELECT {SESSION_COLUMNS} FROM live_sessions WHERE course_id = $1 \
         ORDER BY scheduled_start DESC"
    )))
    .bind(course_id)
    .fetch_all(pool)
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_session(
    pool: &PgPool,
    id: Uuid,
    title: Option<&str>,
    description: Option<&str>,
    scheduled_start: Option<DateTime<Utc>>,
    scheduled_end: Option<DateTime<Utc>>,
) -> Result<LiveSession> {
    Ok(sqlx::query_as::<_, LiveSession>(AssertSqlSafe(format!(
        "UPDATE live_sessions SET \
            title = COALESCE($2, title), \
            description = COALESCE($3, description), \
            scheduled_start = COALESCE($4, scheduled_start), \
            scheduled_end = COALESCE($5, scheduled_end), \
            updated_at = now() \
         WHERE id = $1 \
         RETURNING {SESSION_COLUMNS}"
    )))
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(scheduled_start)
    .bind(scheduled_end)
    .fetch_one(pool)
    .await?)
}

pub async fn set_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    started_at: Option<DateTime<Utc>>,
    ended_at: Option<DateTime<Utc>>,
) -> Result<LiveSession> {
    Ok(sqlx::query_as::<_, LiveSession>(AssertSqlSafe(format!(
        "UPDATE live_sessions SET \
            status = $2, \
            started_at = COALESCE($3, started_at), \
            ended_at = COALESCE($4, ended_at), \
            updated_at = now() \
         WHERE id = $1 \
         RETURNING {SESSION_COLUMNS}"
    )))
    .bind(id)
    .bind(status)
    .bind(started_at)
    .bind(ended_at)
    .fetch_one(pool)
    .await?)
}

// ---------------------------------------------------------------------------
// Attendance
// ---------------------------------------------------------------------------

pub async fn record_join(
    pool: &PgPool,
    session_id: Uuid,
    user_id: Uuid,
    role: &str,
) -> Result<Attendance> {
    Ok(sqlx::query_as::<_, Attendance>(
        "INSERT INTO live_attendance (session_id, user_id, role) \
         VALUES ($1, $2, $3) \
         RETURNING id, session_id, user_id, role, joined_at, left_at, duration_seconds",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(role)
    .fetch_one(pool)
    .await?)
}

pub async fn record_leave(pool: &PgPool, attendance_id: Uuid) -> Result<Attendance> {
    Ok(sqlx::query_as::<_, Attendance>(
        "UPDATE live_attendance SET \
            left_at = now(), \
            duration_seconds = EXTRACT(EPOCH FROM (now() - joined_at))::int \
         WHERE id = $1 \
         RETURNING id, session_id, user_id, role, joined_at, left_at, duration_seconds",
    )
    .bind(attendance_id)
    .fetch_one(pool)
    .await?)
}

pub async fn list_attendance(pool: &PgPool, session_id: Uuid) -> Result<Vec<Attendance>> {
    Ok(sqlx::query_as::<_, Attendance>(
        "SELECT id, session_id, user_id, role, joined_at, left_at, duration_seconds \
         FROM live_attendance WHERE session_id = $1 ORDER BY joined_at",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?)
}

// ---------------------------------------------------------------------------
// Chat
// ---------------------------------------------------------------------------

pub async fn insert_chat_message(
    pool: &PgPool,
    session_id: Uuid,
    sender_id: Uuid,
    body: &str,
) -> Result<ChatMessage> {
    Ok(sqlx::query_as::<_, ChatMessage>(
        "INSERT INTO live_chat_messages (session_id, sender_id, body) \
         VALUES ($1, $2, $3) \
         RETURNING id, session_id, sender_id, body, created_at",
    )
    .bind(session_id)
    .bind(sender_id)
    .bind(body)
    .fetch_one(pool)
    .await?)
}

pub async fn list_chat_messages(
    pool: &PgPool,
    session_id: Uuid,
    limit: i64,
) -> Result<Vec<ChatMessage>> {
    Ok(sqlx::query_as::<_, ChatMessage>(
        "SELECT id, session_id, sender_id, body, created_at \
         FROM live_chat_messages WHERE session_id = $1 \
         ORDER BY created_at DESC LIMIT $2",
    )
    .bind(session_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map(|mut rows| {
        rows.reverse();
        rows
    })?)
}

// ---------------------------------------------------------------------------
// Resources & recordings
// ---------------------------------------------------------------------------

pub async fn insert_resource(
    pool: &PgPool,
    session_id: Uuid,
    name: &str,
    url: &str,
    kind: &str,
    created_by: Uuid,
) -> Result<SessionResource> {
    Ok(sqlx::query_as::<_, SessionResource>(
        "INSERT INTO live_session_resources (session_id, name, url, kind, created_by) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, session_id, name, url, kind, created_by, created_at",
    )
    .bind(session_id)
    .bind(name)
    .bind(url)
    .bind(kind)
    .bind(created_by)
    .fetch_one(pool)
    .await?)
}

pub async fn list_resources(pool: &PgPool, session_id: Uuid) -> Result<Vec<SessionResource>> {
    Ok(sqlx::query_as::<_, SessionResource>(
        "SELECT id, session_id, name, url, kind, created_by, created_at \
         FROM live_session_resources WHERE session_id = $1 ORDER BY created_at",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_recording(
    pool: &PgPool,
    session_id: Uuid,
    url: &str,
    duration_seconds: Option<i32>,
    size_bytes: Option<i64>,
    created_by: Uuid,
) -> Result<Recording> {
    Ok(sqlx::query_as::<_, Recording>(
        "INSERT INTO live_recordings (session_id, url, duration_seconds, size_bytes, created_by) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, session_id, url, duration_seconds, size_bytes, created_by, created_at",
    )
    .bind(session_id)
    .bind(url)
    .bind(duration_seconds)
    .bind(size_bytes)
    .bind(created_by)
    .fetch_one(pool)
    .await?)
}

// ---------------------------------------------------------------------------
// Cross-plugin helpers (shared `courses` / `course_enrollments` tables)
// ---------------------------------------------------------------------------

/// `(owner_id)` of the course, or `None` if course_management hasn't
/// initialized its tables yet (defer the check, like the enrollments plugin).
pub async fn course_owner(pool: &PgPool, course_id: Uuid) -> Result<Option<Uuid>> {
    let exists: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public.courses')::text")
            .fetch_optional(pool)
            .await?;
    if exists.is_none() {
        return Ok(None);
    }
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT owner_id FROM courses WHERE id = $1")
        .bind(course_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(o,)| o))
}

pub async fn is_course_instructor(pool: &PgPool, course_id: Uuid, user_id: Uuid) -> Result<bool> {
    let exists: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public.course_instructors')::text")
            .fetch_optional(pool)
            .await?;
    if exists.is_none() {
        return Ok(false);
    }
    let authorized: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM course_instructors WHERE course_id = $1 AND user_id = $2)",
    )
    .bind(course_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(authorized)
}

/// Whether `user_id` has an *active* enrollment in `course_id`. Defers to
/// `true` if the enrollments plugin hasn't initialized its tables yet.
pub async fn has_active_enrollment(pool: &PgPool, course_id: Uuid, user_id: Uuid) -> Result<bool> {
    let exists: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public.course_enrollments')::text")
            .fetch_optional(pool)
            .await?;
    if exists.is_none() {
        return Ok(true);
    }
    let active: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM course_enrollments \
         WHERE course_id = $1 AND student_id = $2 AND state = 'active')",
    )
    .bind(course_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(active)
}
