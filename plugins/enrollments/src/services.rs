//! Enrollment business logic.
//!
//! The `course_management` plugin owns `courses`/`course_instructors` and the
//! `auth` plugin owns `users` — all three live in the same shared PostgreSQL
//! database, so the enrollment service enforces cross-plugin rules directly:
//!
//! * a course must exist (and usually be published) to enroll into;
//! * only instructors/admins may enroll others or approve;
//! * only **active** enrollments grant access to course content.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{
    can_transition, generate_invite_token, valid_token_shape, EnrollmentSource, EnrollmentState,
};
use crate::models::Enrollment;
use crate::repository;

// ---------------------------------------------------------------------------
// Course existence / access helpers (shared `courses` table)
// ---------------------------------------------------------------------------

/// Verify the course exists (any status). Returns `(course_id, owner_id)`;
/// `owner_id` is `nil` when course_management has not initialized yet.
pub async fn require_course(pool: &PgPool, course_id: Uuid) -> Result<(Uuid, Uuid)> {
    let exists: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public.courses')::text")
            .fetch_optional(pool)
            .await?;
    if exists.is_none() {
        // course_management has not initialized yet — defer the check.
        return Ok((course_id, Uuid::nil()));
    }
    let row: Option<(Uuid, Uuid)> =
        sqlx::query_as("SELECT id, owner_id FROM courses WHERE id = $1")
            .bind(course_id)
            .fetch_optional(pool)
            .await?;
    row.map(|(id, owner)| (id, owner))
        .ok_or_else(|| anyhow!("course not found"))
}

/// True when the caller manages the course (owner, course_instructor or admin).
pub async fn is_manager(
    pool: &PgPool,
    user_id: Uuid,
    is_admin: bool,
    course_id: Uuid,
) -> Result<bool> {
    if is_admin {
        return Ok(true);
    }
    let exists: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public.courses')::text")
            .fetch_optional(pool)
            .await?;
    if exists.is_none() {
        return Ok(false);
    }
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT owner_id FROM courses WHERE id = $1")
            .bind(course_id)
            .fetch_optional(pool)
            .await?;
    if owner.map(|(o,)| o) == Some(user_id) {
        return Ok(true);
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

// ---------------------------------------------------------------------------
// Enrollment actions
// ---------------------------------------------------------------------------

/// The final state for a new enrollment given the source.
fn initial_state(source: EnrollmentSource) -> EnrollmentState {
    match source {
        // Manager-initiated enrollments are immediate; self-service needs
        // approval. Invite joins count as manager-sanctioned.
        EnrollmentSource::Self_ => EnrollmentState::Pending,
        _ => EnrollmentState::Active,
    }
}

/// Convert a joined enrollment row into its wire representation.
pub fn to_enrollment_view(e: &Enrollment) -> crate::dto::EnrollmentView {
    crate::dto::EnrollmentView {
        id: e.id,
        course_id: e.course_id,
        student: crate::dto::StudentView {
            id: e.student_id,
            username: e.username.clone(),
            full_name: e.full_name.clone(),
            email: e.email.clone(),
        },
        state: e.state.clone(),
        source: e.source.clone(),
        expires_at: e.expires_at,
        created_at: e.created_at,
        updated_at: e.updated_at,
    }
}

/// Self-enrollment and invite joins bypass the manager check (the caller is
/// the student), but the course must exist and — for plain self-enrollment —
/// be published. Manager-initiated enrollments only need the course to exist.
async fn course_admission(pool: &PgPool, course_id: Uuid, source: EnrollmentSource) -> Result<()> {
    let (id, _) = require_course(pool, course_id).await?;
    if id == Uuid::nil() {
        // course_management not initialized yet — defer.
        return Ok(());
    }
    if matches!(source, EnrollmentSource::Self_) {
        let published: Option<bool> =
            sqlx::query_scalar("SELECT (status = 'published') FROM courses WHERE id = $1")
                .bind(course_id)
                .fetch_optional(pool)
                .await?;
        match published {
            Some(true) => {}
            Some(false) => return Err(anyhow!("course is not open for enrollment")),
            None => return Err(anyhow!("course not found")),
        }
    }
    Ok(())
}

/// Enroll one student (core primitive used by every entry point).
pub async fn enroll_student(
    pool: &PgPool,
    student_id: Uuid,
    course_id: Uuid,
    source: EnrollmentSource,
    actor: Option<Uuid>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<crate::dto::EnrollmentView> {
    course_admission(pool, course_id, source).await?;
    let state = initial_state(source);
    let enrollment_id = repository::upsert_enrollment(
        pool,
        course_id,
        student_id,
        state.as_str(),
        source.as_str(),
        actor,
        expires_at,
    )
    .await?;
    repository::insert_history(
        pool,
        enrollment_id,
        None,
        state.as_str(),
        actor,
        Some("enrolled"),
    )
    .await?;
    let enrollment = repository::find_enrollment_by_id(pool, enrollment_id)
        .await?
        .ok_or_else(|| anyhow!("enrollment vanished after creation"))?;
    Ok(to_enrollment_view(&enrollment))
}

/// Instructor/admin manually enrolls a student by username or email.
pub async fn enroll_by_manager(
    pool: &PgPool,
    actor_id: Uuid,
    is_admin: bool,
    course_id: Uuid,
    student_identifier: &str,
    expires_at: Option<DateTime<Utc>>,
) -> Result<crate::dto::EnrollmentView> {
    if !is_manager(pool, actor_id, is_admin, course_id).await? {
        return Err(anyhow!("forbidden: you do not manage this course"));
    }
    let (student_id, _, _, _) = repository::require_user(pool, student_identifier).await?;
    let source = if is_admin {
        EnrollmentSource::Admin
    } else {
        EnrollmentSource::Instructor
    };
    enroll_student(pool, student_id, course_id, source, Some(actor_id), expires_at).await
}

// ---------------------------------------------------------------------------
// Student actions (self-enrollment, invite joins)
// ---------------------------------------------------------------------------

/// Student self-enrollment: lands in `pending` until a manager approves it.
pub async fn self_enroll(
    pool: &PgPool,
    user_id: Uuid,
    course_id: Uuid,
    expires_at: Option<DateTime<Utc>>,
) -> Result<crate::dto::EnrollmentView> {
    enroll_student(
        pool,
        user_id,
        course_id,
        EnrollmentSource::Self_,
        Some(user_id),
        expires_at,
    )
    .await
}

/// Join a course through an invite link (the "course link" flow). The join is
/// manager-sanctioned, so the enrollment is immediately `active`.
pub async fn join_by_invite(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
) -> Result<crate::dto::EnrollmentView> {
    if !valid_token_shape(token) {
        return Err(anyhow!("invalid invite link"));
    }
    let invite = repository::find_invite_by_token(pool, token)
        .await?
        .ok_or_else(|| anyhow!("invalid invite link"))?;
    if !invite.active {
        return Err(anyhow!("this invite link has been revoked"));
    }
    let now = Utc::now();
    if let Some(exp) = invite.expires_at {
        if exp <= now {
            return Err(anyhow!("this invite link has expired"));
        }
    }
    if let Some(max) = invite.max_uses {
        if invite.uses >= max {
            return Err(anyhow!("this invite link has been fully used"));
        }
    }
    let view = enroll_student(
        pool,
        user_id,
        invite.course_id,
        EnrollmentSource::Invite,
        Some(user_id),
        None,
    )
    .await?;
    repository::consume_invite(pool, invite.id).await?;
    Ok(view)
}

/// Bulk enrollment: enroll many students at once (instructor/admin).
pub async fn bulk_enroll(
    pool: &PgPool,
    actor_id: Uuid,
    is_admin: bool,
    course_id: Uuid,
    students: &[String],
    expires_at: Option<DateTime<Utc>>,
) -> Result<crate::dto::BulkEnrollView> {
    if !is_manager(pool, actor_id, is_admin, course_id).await? {
        return Err(anyhow!("forbidden: you do not manage this course"));
    }
    let mut results = Vec::new();
    for identifier in students {
        let entry = match repository::require_user(pool, identifier).await {
            Ok((student_id, _, _, _)) => {
                match enroll_student(
                    pool,
                    student_id,
                    course_id,
                    EnrollmentSource::Bulk,
                    Some(actor_id),
                    expires_at,
                )
                .await
                {
                    Ok(view) => crate::dto::BulkEntryResult {
                        student: identifier.clone(),
                        ok: true,
                        message: "enrolled".into(),
                        enrollment: Some(view),
                    },
                    Err(err) => crate::dto::BulkEntryResult {
                        student: identifier.clone(),
                        ok: false,
                        message: err.to_string(),
                        enrollment: None,
                    },
                }
            }
            Err(err) => crate::dto::BulkEntryResult {
                student: identifier.clone(),
                ok: false,
                message: err.to_string(),
                enrollment: None,
            },
        };
        results.push(entry);
    }
    let enrolled = results.iter().filter(|r| r.ok).count();
    let failed = results.len() - enrolled;
    Ok(crate::dto::BulkEnrollView { course_id, enrolled, failed, results })
}

// ---------------------------------------------------------------------------
// State changes (approve / suspend / cancel / complete / re-enroll)
// ---------------------------------------------------------------------------

/// Change an enrollment's state with full validation + history.
pub async fn change_state(
    pool: &PgPool,
    enrollment_id: Uuid,
    to: EnrollmentState,
    actor_id: Uuid,
    is_admin: bool,
    reason: Option<&str>,
) -> Result<crate::dto::EnrollmentView> {
    let enrollment = repository::get_enrollment(pool, enrollment_id)
        .await?
        .ok_or_else(|| anyhow!("enrollment not found"))?;

    let manager = is_manager(pool, actor_id, is_admin, enrollment.course_id).await?;
    let self_cancel = to == EnrollmentState::Cancelled && enrollment.student_id == actor_id;

    // Only managers may approve/suspend/complete; students may only cancel
    // their own enrollment (withdraw).
    if !manager && !self_cancel {
        return Err(anyhow!("forbidden: you cannot modify this enrollment"));
    }

    // Approving a pending enrollment is manager-only.
    if to == EnrollmentState::Active && enrollment.state == "pending" && !manager {
        return Err(anyhow!("forbidden: only the course's instructors can approve"));
    }

    let from = EnrollmentState::from_str(&enrollment.state)
        .ok_or_else(|| anyhow!("corrupt enrollment state '{}'", enrollment.state))?;

    // Re-enrollment: allow valid restarts and block impossible jumps.
    if !can_transition(from, to) {
        return Err(anyhow!(
            "cannot move enrollment from '{}' to '{}'",
            from.as_str(),
            to.as_str()
        ));
    }

    // Re-enrolling an expired/cancelled/completed enrollment resets the clock.
    let now = Utc::now();
    let (approved_by, approved_at, completed_at, suspended_at, cancelled_at) = match to {
        EnrollmentState::Active => (Some(actor_id), Some(now), None, None, None),
        EnrollmentState::Completed => (None, None, Some(now), None, None),
        EnrollmentState::Suspended => (None, None, None, Some(now), None),
        EnrollmentState::Cancelled => (None, None, None, None, Some(now)),
        _ => (None, None, None, None, None),
    };
    repository::update_enrollment_state_full(
        pool,
        enrollment_id,
        to.as_str(),
        approved_by,
        approved_at,
        completed_at,
        suspended_at,
        cancelled_at,
    )
    .await?;
    repository::insert_history(
        pool,
        enrollment_id,
        Some(from.as_str()),
        to.as_str(),
        Some(actor_id),
        reason,
    )
    .await?;

    repository::get_enrollment(pool, enrollment_id)
        .await?
        .map(|e| to_enrollment_view(&e))
        .ok_or_else(|| anyhow!("enrollment vanished"))
}

/// Expire every active enrollment whose deadline has passed. Returns the
/// number of enrollments expired. Called opportunistically by the plugin.
pub async fn sweep_expired(pool: &PgPool) -> Result<u64> {
    let stale = repository::list_expired_active(pool, Utc::now()).await?;
    let mut count = 0u64;
    for e in stale {
        repository::update_enrollment_state_full(
            pool,
            e.id,
            EnrollmentState::Expired.as_str(),
            None,
            None,
            None,
            None,
            None,
        )
        .await?;
        repository::insert_history(
            pool,
            e.id,
            Some(EnrollmentState::Active.as_str()),
            EnrollmentState::Expired.as_str(),
            None,
            Some("enrollment deadline passed"),
        )
        .await?;
        count += 1;
    }
    Ok(count)
}

// ---------------------------------------------------------------------------
// Access checks (used by assignments / lessons to gate content)
// ---------------------------------------------------------------------------

/// Whether `user_id` may access `course_id`'s content (lessons, assignments).
///
/// Access is granted when the user has an **active** enrollment or manages
/// the course (owner / instructor / admin).
pub async fn check_content_access(
    pool: &PgPool,
    user_id: Uuid,
    is_admin: bool,
    course_id: Uuid,
) -> Result<crate::dto::AccessView> {
    let manager = is_manager(pool, user_id, is_admin, course_id).await?;
    let enrollment = repository::find_enrollment(pool, course_id, user_id).await?;
    let state = enrollment.map(|e| e.state);
    let allowed = manager
        || state
            .as_deref()
            .and_then(EnrollmentState::from_str)
            .is_some_and(|s| s.grants_access());
    Ok(crate::dto::AccessView {
        course_id,
        allowed,
        state,
        manager,
    })
}

// ---------------------------------------------------------------------------
// Rosters & history
// ---------------------------------------------------------------------------

/// The student roster of a course (manager-only, admins included).
pub async fn course_roster(
    pool: &PgPool,
    actor_id: Uuid,
    is_admin: bool,
    course_id: Uuid,
    state: Option<&str>,
) -> Result<Vec<crate::dto::EnrollmentView>> {
    if !is_manager(pool, actor_id, is_admin, course_id).await? {
        return Err(anyhow!("forbidden: you do not manage this course"));
    }
    Ok(repository::list_course_enrollments(pool, course_id, state)
        .await?
        .iter()
        .map(to_enrollment_view)
        .collect())
}

/// The caller's own enrollments (student view of "my courses").
pub async fn my_enrollments(
    pool: &PgPool,
    student_id: Uuid,
) -> Result<Vec<crate::dto::EnrollmentView>> {
    Ok(repository::list_student_enrollments(pool, student_id)
        .await?
        .iter()
        .map(to_enrollment_view)
        .collect())
}

/// A single enrollment (manager or the owning student).
pub async fn get_enrollment(
    pool: &PgPool,
    actor_id: Uuid,
    is_admin: bool,
    enrollment_id: Uuid,
) -> Result<crate::dto::EnrollmentView> {
    let enrollment = repository::get_enrollment(pool, enrollment_id)
        .await?
        .ok_or_else(|| anyhow!("enrollment not found"))?;
    let manager = is_manager(pool, actor_id, is_admin, enrollment.course_id).await?;
    if !manager && enrollment.student_id != actor_id {
        return Err(anyhow!("forbidden"));
    }
    Ok(to_enrollment_view(&enrollment))
}

/// The full state history of an enrollment (manager or the owning student).
pub async fn enrollment_history(
    pool: &PgPool,
    actor_id: Uuid,
    is_admin: bool,
    enrollment_id: Uuid,
) -> Result<Vec<crate::dto::HistoryView>> {
    let enrollment = repository::get_enrollment(pool, enrollment_id)
        .await?
        .ok_or_else(|| anyhow!("enrollment not found"))?;
    let manager = is_manager(pool, actor_id, is_admin, enrollment.course_id).await?;
    if !manager && enrollment.student_id != actor_id {
        return Err(anyhow!("forbidden"));
    }
    Ok(repository::list_history(pool, enrollment_id)
        .await?
        .into_iter()
        .map(|h| crate::dto::HistoryView {
            id: h.id,
            enrollment_id: h.enrollment_id,
            from_state: h.from_state,
            to_state: h.to_state,
            changed_by: h.changed_by,
            reason: h.reason,
            created_at: h.created_at,
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Invites (course links)
// ---------------------------------------------------------------------------

/// Create a shareable invite link for a course (manager-only).
pub async fn create_invite(
    pool: &PgPool,
    actor_id: Uuid,
    is_admin: bool,
    course_id: Uuid,
    max_uses: Option<i32>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<crate::dto::InviteView> {
    if !is_manager(pool, actor_id, is_admin, course_id).await? {
        return Err(anyhow!("forbidden: you do not manage this course"));
    }
    require_course(pool, course_id).await?;
    let token = generate_invite_token();
    let invite = repository::insert_invite(pool, course_id, &token, actor_id, max_uses, expires_at)
        .await?;
    Ok(crate::dto::InviteView {
        id: invite.id,
        course_id: invite.course_id,
        token: invite.token,
        created_by: invite.created_by,
        max_uses: invite.max_uses,
        uses: invite.uses,
        expires_at: invite.expires_at,
        active: invite.active,
        created_at: invite.created_at,
    })
}

/// List a course's invites (manager-only).
pub async fn course_invites(
    pool: &PgPool,
    actor_id: Uuid,
    is_admin: bool,
    course_id: Uuid,
) -> Result<Vec<crate::dto::InviteView>> {
    if !is_manager(pool, actor_id, is_admin, course_id).await? {
        return Err(anyhow!("forbidden: you do not manage this course"));
    }
    Ok(repository::list_invites(pool, course_id)
        .await?
        .into_iter()
        .map(|i| crate::dto::InviteView {
            id: i.id,
            course_id: i.course_id,
            token: i.token,
            created_by: i.created_by,
            max_uses: i.max_uses,
            uses: i.uses,
            expires_at: i.expires_at,
            active: i.active,
            created_at: i.created_at,
        })
        .collect())
}

/// Revoke an invite link (manager-only).
pub async fn revoke_invite(
    pool: &PgPool,
    actor_id: Uuid,
    is_admin: bool,
    invite_id: Uuid,
) -> Result<crate::dto::InviteView> {
    let invite = repository::find_invite_by_id(pool, invite_id)
        .await?
        .ok_or_else(|| anyhow!("invite not found"))?;
    if !is_manager(pool, actor_id, is_admin, invite.course_id).await? {
        return Err(anyhow!("forbidden: you do not manage this course"));
    }
    let invite = repository::revoke_invite(pool, invite_id).await?;
    Ok(crate::dto::InviteView {
        id: invite.id,
        course_id: invite.course_id,
        token: invite.token,
        created_by: invite.created_by,
        max_uses: invite.max_uses,
        uses: invite.uses,
        expires_at: invite.expires_at,
        active: invite.active,
        created_at: invite.created_at,
    })
}

/// Re-enroll a cancelled / expired / completed enrollment.
///
/// * Managers reactivate the enrollment directly (`→ active`).
/// * Students re-apply for approval (`→ pending`).
pub async fn re_enroll(
    pool: &PgPool,
    actor_id: Uuid,
    is_admin: bool,
    enrollment_id: Uuid,
) -> Result<crate::dto::EnrollmentView> {
    let enrollment = repository::get_enrollment(pool, enrollment_id)
        .await?
        .ok_or_else(|| anyhow!("enrollment not found"))?;

    let from = EnrollmentState::from_str(&enrollment.state)
        .ok_or_else(|| anyhow!("corrupt enrollment state '{}'", enrollment.state))?;
    if !from.can_re_enroll() {
        return Err(anyhow!(
            "an enrollment in state '{}' cannot be re-enrolled",
            from.as_str()
        ));
    }

    let manager = is_manager(pool, actor_id, is_admin, enrollment.course_id).await?;
    if !manager && enrollment.student_id != actor_id {
        return Err(anyhow!("forbidden: you cannot re-enroll this student"));
    }

    // We've performed our own permission check above; delegate the validated
    // transition (and history bookkeeping) to change_state.
    let to = if manager {
        EnrollmentState::Active
    } else {
        EnrollmentState::Pending
    };
    change_state(pool, enrollment_id, to, actor_id, true, Some("re-enrollment")).await
}
