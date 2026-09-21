//! Live-session business logic: scheduling, lifecycle, attendance, chat,
//! resources and authorization against the shared `courses` /
//! `course_enrollments` tables (owned by course_management / enrollments).

use anyhow::{anyhow, Result};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::*;
use crate::models::LiveSession;
use crate::repository;

fn to_view(s: &LiveSession) -> SessionView {
    SessionView {
        id: s.id,
        course_id: s.course_id,
        title: s.title.clone(),
        description: s.description.clone(),
        host_id: s.host_id,
        provider: s.provider.clone(),
        external_url: s.external_url.clone(),
        scheduled_start: s.scheduled_start,
        scheduled_end: s.scheduled_end,
        started_at: s.started_at,
        ended_at: s.ended_at,
        status: s.status.clone(),
        created_at: s.created_at,
        updated_at: s.updated_at,
    }
}

/// True when the caller manages the course (admin, owner, or course instructor).
pub async fn is_manager(pool: &PgPool, user_id: Uuid, is_admin: bool, course_id: Uuid) -> Result<bool> {
    if is_admin {
        return Ok(true);
    }
    if let Some(owner) = repository::course_owner(pool, course_id).await? {
        if owner == user_id {
            return Ok(true);
        }
    }
    repository::is_course_instructor(pool, course_id, user_id).await
}

pub async fn schedule(
    pool: &PgPool,
    actor: Uuid,
    is_admin: bool,
    req: ScheduleRequest,
) -> Result<SessionView> {
    if !is_manager(pool, actor, is_admin, req.course_id).await? {
        return Err(anyhow!("forbidden: you do not manage this course"));
    }
    if req.scheduled_end <= req.scheduled_start {
        return Err(anyhow!("scheduled_end must be after scheduled_start"));
    }
    let provider = req.provider.to_lowercase();
    if !["webrtc", "jitsi", "zoom", "teams"].contains(&provider.as_str()) {
        return Err(anyhow!("unsupported provider '{provider}'"));
    }
    if provider != "webrtc" && req.external_url.as_deref().unwrap_or("").is_empty() {
        return Err(anyhow!("external_url is required for provider '{provider}'"));
    }
    let session = repository::insert_session(
        pool,
        req.course_id,
        req.title.trim(),
        &req.description,
        actor,
        &provider,
        req.external_url.as_deref(),
        req.scheduled_start,
        req.scheduled_end,
    )
    .await?;
    Ok(to_view(&session))
}

pub async fn update(
    pool: &PgPool,
    actor: Uuid,
    is_admin: bool,
    req: UpdateRequest,
) -> Result<SessionView> {
    let session = repository::find_session(pool, req.session_id)
        .await?
        .ok_or_else(|| anyhow!("session not found"))?;
    if !is_manager(pool, actor, is_admin, session.course_id).await? {
        return Err(anyhow!("forbidden: you do not manage this course"));
    }
    let updated = repository::update_session(
        pool,
        req.session_id,
        req.title.as_deref(),
        req.description.as_deref(),
        req.scheduled_start,
        req.scheduled_end,
    )
    .await?;
    Ok(to_view(&updated))
}

/// Expire any scheduled sessions whose `scheduled_end` has passed
/// (`status: scheduled → expired`). Runs before reads and joins so expiry is
/// lazy — no background worker. Failures are non-fatal (best-effort sweep).
async fn sweep_expired(pool: &PgPool) {
    if let Err(err) = repository::expire_stale_sessions(pool).await {
        tracing::warn!("live_classes: failed to sweep expired sessions: {err}");
    }
}

pub async fn list_for_course(pool: &PgPool, course_id: Uuid) -> Result<Vec<SessionView>> {
    sweep_expired(pool).await;
    let rows = repository::list_course_sessions(pool, course_id).await?;
    Ok(rows.iter().map(to_view).collect())
}

pub async fn get(pool: &PgPool, session_id: Uuid) -> Result<SessionView> {
    sweep_expired(pool).await;
    let s = repository::find_session(pool, session_id)
        .await?
        .ok_or_else(|| anyhow!("session not found"))?;
    Ok(to_view(&s))
}

async fn require_session(pool: &PgPool, session_id: Uuid) -> Result<LiveSession> {
    repository::find_session(pool, session_id)
        .await?
        .ok_or_else(|| anyhow!("session not found"))
}

/// Whether `actor` may join (view/attend) a session: managers always can;
/// everyone else needs an active enrollment in the session's course.
async fn can_join(pool: &PgPool, actor: Uuid, is_admin: bool, session: &LiveSession) -> Result<bool> {
    if is_manager(pool, actor, is_admin, session.course_id).await? {
        return Ok(true);
    }
    repository::has_active_enrollment(pool, session.course_id, actor).await
}

/// `live.join`: validate access, record attendance, and return the signaling
/// path the frontend should open a WebSocket to (webrtc sessions only).
pub async fn join(pool: &PgPool, actor: Uuid, is_admin: bool, session_id: Uuid) -> Result<JoinView> {
    sweep_expired(pool).await;
    let session = require_session(pool, session_id).await?;
    if !matches!(session.status.as_str(), "scheduled" | "live") {
        if session.status == "expired" {
            return Err(anyhow!("this session has expired: its scheduled end time has passed"));
        }
        return Err(anyhow!("this session is not currently joinable"));
    }
    let allowed = can_join(pool, actor, is_admin, &session).await?;
    if !allowed {
        return Err(anyhow!("forbidden: you are not enrolled in this course"));
    }
    let manager = is_manager(pool, actor, is_admin, session.course_id).await?;
    let role = if manager || session.host_id == actor { "host" } else { "attendee" };
    let attendance = repository::record_join(pool, session_id, actor, role).await?;

    let signaling_path = if session.provider == "webrtc" {
        Some(format!("/ws/live/{session_id}"))
    } else {
        None
    };
    Ok(JoinView {
        session: to_view(&session),
        allowed,
        role: role.to_string(),
        attendance_id: attendance.id,
        signaling_path,
    })
}

pub async fn start(pool: &PgPool, actor: Uuid, is_admin: bool, session_id: Uuid) -> Result<SessionView> {
    let session = require_session(pool, session_id).await?;
    if !is_manager(pool, actor, is_admin, session.course_id).await? {
        return Err(anyhow!("forbidden: only the course's instructors can start this session"));
    }
    let updated = repository::set_status(pool, session_id, "live", Some(Utc::now()), None).await?;
    Ok(to_view(&updated))
}

pub async fn end(pool: &PgPool, actor: Uuid, is_admin: bool, session_id: Uuid) -> Result<SessionView> {
    let session = require_session(pool, session_id).await?;
    if !is_manager(pool, actor, is_admin, session.course_id).await? {
        return Err(anyhow!("forbidden: only the course's instructors can end this session"));
    }
    let updated = repository::set_status(pool, session_id, "ended", None, Some(Utc::now())).await?;
    Ok(to_view(&updated))
}

pub async fn cancel(pool: &PgPool, actor: Uuid, is_admin: bool, session_id: Uuid) -> Result<SessionView> {
    let session = require_session(pool, session_id).await?;
    if !is_manager(pool, actor, is_admin, session.course_id).await? {
        return Err(anyhow!("forbidden: only the course's instructors can cancel this session"));
    }
    let updated = repository::set_status(pool, session_id, "cancelled", None, None).await?;
    Ok(to_view(&updated))
}

/// Record that a participant left (stamps `left_at` / `duration_seconds`).
pub async fn leave(pool: &PgPool, attendance_id: Uuid) -> Result<()> {
    repository::record_leave(pool, attendance_id).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Attendance
// ---------------------------------------------------------------------------

pub async fn list_attendance(
    pool: &PgPool,
    actor: Uuid,
    is_admin: bool,
    session_id: Uuid,
) -> Result<Vec<AttendanceView>> {
    let session = require_session(pool, session_id).await?;
    if !is_manager(pool, actor, is_admin, session.course_id).await? {
        return Err(anyhow!("forbidden: only the course's instructors can view attendance"));
    }
    let rows = repository::list_attendance(pool, session_id).await?;
    Ok(rows
        .into_iter()
        .map(|a| AttendanceView {
            id: a.id,
            session_id: a.session_id,
            user_id: a.user_id,
            role: a.role,
            joined_at: a.joined_at,
            left_at: a.left_at,
            duration_seconds: a.duration_seconds,
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Chat
// ---------------------------------------------------------------------------

pub async fn send_chat(
    pool: &PgPool,
    actor: Uuid,
    is_admin: bool,
    req: ChatSendRequest,
) -> Result<ChatMessageView> {
    let session = require_session(pool, req.session_id).await?;
    if !can_join(pool, actor, is_admin, &session).await? {
        return Err(anyhow!("forbidden: you are not enrolled in this course"));
    }
    let body = req.body.trim();
    if body.is_empty() {
        return Err(anyhow!("message cannot be empty"));
    }
    let msg = repository::insert_chat_message(pool, req.session_id, actor, body).await?;
    Ok(ChatMessageView {
        id: msg.id,
        session_id: msg.session_id,
        sender_id: msg.sender_id,
        body: msg.body,
        created_at: msg.created_at,
    })
}

pub async fn chat_history(
    pool: &PgPool,
    actor: Uuid,
    is_admin: bool,
    req: ChatHistoryRequest,
) -> Result<Vec<ChatMessageView>> {
    let session = require_session(pool, req.session_id).await?;
    if !can_join(pool, actor, is_admin, &session).await? {
        return Err(anyhow!("forbidden: you are not enrolled in this course"));
    }
    let rows = repository::list_chat_messages(pool, req.session_id, req.limit.unwrap_or(200)).await?;
    Ok(rows
        .into_iter()
        .map(|m| ChatMessageView {
            id: m.id,
            session_id: m.session_id,
            sender_id: m.sender_id,
            body: m.body,
            created_at: m.created_at,
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Resources & recordings
// ---------------------------------------------------------------------------

pub async fn attach_resource(
    pool: &PgPool,
    actor: Uuid,
    is_admin: bool,
    req: AttachResourceRequest,
) -> Result<ResourceView> {
    let session = require_session(pool, req.session_id).await?;
    if !is_manager(pool, actor, is_admin, session.course_id).await? {
        return Err(anyhow!("forbidden: only the course's instructors can attach resources"));
    }
    let r = repository::insert_resource(pool, req.session_id, req.name.trim(), &req.url, &req.kind, actor)
        .await?;
    Ok(ResourceView {
        id: r.id,
        session_id: r.session_id,
        name: r.name,
        url: r.url,
        kind: r.kind,
        created_by: r.created_by,
        created_at: r.created_at,
    })
}

pub async fn list_resources(
    pool: &PgPool,
    actor: Uuid,
    is_admin: bool,
    session_id: Uuid,
) -> Result<Vec<ResourceView>> {
    let session = require_session(pool, session_id).await?;
    if !can_join(pool, actor, is_admin, &session).await? {
        return Err(anyhow!("forbidden: you are not enrolled in this course"));
    }
    let rows = repository::list_resources(pool, session_id).await?;
    Ok(rows
        .into_iter()
        .map(|r| ResourceView {
            id: r.id,
            session_id: r.session_id,
            name: r.name,
            url: r.url,
            kind: r.kind,
            created_by: r.created_by,
            created_at: r.created_at,
        })
        .collect())
}

pub async fn save_recording(
    pool: &PgPool,
    actor: Uuid,
    is_admin: bool,
    req: SaveRecordingRequest,
) -> Result<RecordingView> {
    let session = require_session(pool, req.session_id).await?;
    if !is_manager(pool, actor, is_admin, session.course_id).await? {
        return Err(anyhow!("forbidden: only the course's instructors can save a recording"));
    }
    let r = repository::insert_recording(
        pool,
        req.session_id,
        &req.url,
        req.duration_seconds,
        req.size_bytes,
        actor,
    )
    .await?;
    Ok(RecordingView {
        id: r.id,
        session_id: r.session_id,
        url: r.url,
        duration_seconds: r.duration_seconds,
        size_bytes: r.size_bytes,
        created_by: r.created_by,
        created_at: r.created_at,
    })
}
