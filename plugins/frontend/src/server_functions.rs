//! Dioxus "server functions".
//!
//! Each server function is an **Axum** endpoint on the fullstack server. This
//! module is compiled on both the client and the server:
//!
//! * on the **client**, `#[get]` generates a stub that calls the endpoint over HTTP;
//! * on the **server**, the function bodies run and may use SQLx through
//!   [`crate::db`] (SQLx is server-only).
//!
//! See the official docs: https://dioxuslabs.com/learn/0.7/server/fullstack/

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::assignments_types::{
    AssignmentInput, AssignmentView, DownloadView, FileUploadInput, GradeHistoryView, GradeView,
    LinkInput, ResourceView, SubmissionView,
};
use crate::course_types::{CourseInput, CourseView};
use crate::enrollment_types::{
    AccessView, BulkEnrollView, EnrollmentView, HistoryView, InviteView,
};
#[cfg(feature = "server")]
use crate::content_bridge::{
    create_item, create_lesson, create_module, delete_file, delete_item, delete_lesson, delete_module,
    download_file, get_file, get_item, get_lesson, get_module, list_files, list_items, list_lessons,
    list_modules, reorder_items, reorder_lessons, reorder_modules, set_lesson_status,
    set_module_status, update_item, update_lesson, update_module, upload_file,
};
use crate::content_types::{
    ContentFileView, ContentItemView, ContentLessonView, ContentModuleView, CreateContentItemInput,
    CreateLessonInput, CreateModuleInput, FileDownloadView, UpdateContentItemInput, UpdateLessonInput,
    UpdateModuleInput,
};
use crate::live_types::{
    LiveAttendanceView, LiveChatMessageView, LiveJoinView, LiveResourceView, LiveSessionView,
};

/// `/api/health` — check that the server is up and that PostgreSQL is reachable
/// via the SQLx pool.
#[get("/api/health")]
pub async fn health() -> anyhow::Result<Health> {
    let database = if crate::db::ping().await {
        "connected".to_string()
    } else {
        "unavailable".to_string()
    };

    Ok(Health {
        status: "ok".to_string(),
        database,
    })
}

/// Validates an opaque Google OAuth session and returns the signed-in profile.
#[post("/api/auth/verify")]
pub async fn auth_verify(token: String) -> anyhow::Result<AuthSession> {
    let user = crate::google_classroom_bridge::verify_session(&token)?;
    Ok(AuthSession {
        token,
        user: AuthUser {
            id: user.id,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            active: user.active,
            roles: user.roles,
        },
    })
}


/// `/api/courses` — read rows from the `courses` table using SQLx.
///
/// Requires the E-Lupane migrations to have created the `courses` table. If the
/// table does not exist yet (migrations are currently empty) this returns a
/// helpful error rather than failing the whole server.
#[get("/api/courses")]
pub async fn list_courses() -> anyhow::Result<Vec<Course>> {
    let pool =
        crate::db::pool().ok_or_else(|| anyhow::anyhow!("database not available"))?;

    // `sqlx::Row` gives us column access via `row.get("...")`.
    use sqlx::Row as _;

    let rows = sqlx::query(
        "SELECT id::bigint AS id, course_code, course_name, COALESCE(active, false) AS active \
         FROM courses ORDER BY course_name LIMIT 100",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|row| Course {
            id: row.get("id"),
            course_code: row.get("course_code"),
            course_name: row.get("course_name"),
            active: row.get("active"),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub status: String,
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: i64,
    pub course_code: String,
    pub course_name: String,
    pub active: bool,
}

// ---------------------------------------------------------------------------
// Authentication (backed by the auth plugin over the Redis messaging layer)
// ---------------------------------------------------------------------------

/// Authenticated user, as reported by the auth plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub active: bool,
    pub roles: Vec<String>,
}

/// A signed-in session: the bearer token + the user it belongs to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub token: String,
    pub user: AuthUser,
}

/// Convert the auth plugin's `UserView` into the client-facing `AuthUser`.
/// Server-only: server-fn bodies (its only callers) are stripped on the client.
#[cfg(feature = "server")]
fn to_auth_user(view: crate::auth_bridge::UserView) -> AuthUser {
    AuthUser {
        id: view.id,
        username: view.username,
        email: view.email,
        full_name: view.full_name,
        active: view.active,
        roles: view.roles,
    }
}

/// `/api/auth/login` — authenticate via the auth plugin (Redis RPC), returning
/// a token the client stores in `localStorage` for session restore.
#[post("/api/auth/login")]
pub async fn auth_login(username: String, password: String) -> anyhow::Result<AuthSession> {
    let (token, user) = crate::auth_bridge::login(&username, &password).await?;
    Ok(AuthSession {
        token,
        user: to_auth_user(user),
    })
}

/// `/api/auth/verify` — validate a stored token and rebuild the session.
#[post("/api/auth/verify")]
pub async fn auth_verify(token: String) -> anyhow::Result<AuthSession> {
    let user = crate::auth_bridge::verify(&token)
        .await?
        .ok_or_else(|| anyhow::anyhow!("session expired or invalid"))?;
    Ok(AuthSession {
        token,
        user: to_auth_user(user),
    })
}

// ---------------------------------------------------------------------------
// Assignments (backed by the assignments plugin over the Redis messaging layer)
// ---------------------------------------------------------------------------

/// `/api/assignments/list` — all assignments visible to the caller.
#[post("/api/assignments/list")]
pub async fn assignments_list(token: String) -> anyhow::Result<Vec<AssignmentView>> {
    crate::assignments_bridge::list_assignments(&token).await
}

/// `/api/assignments/get` — a single assignment.
#[post("/api/assignments/get")]
pub async fn assignments_get(
    token: String,
    assignment_id: uuid::Uuid,
) -> anyhow::Result<AssignmentView> {
    crate::assignments_bridge::get_assignment(&token, assignment_id).await
}

/// `/api/assignments/create` — instructor/admin creates an assignment.
#[post("/api/assignments/create")]
pub async fn assignments_create(
    token: String,
    input: AssignmentInput,
) -> anyhow::Result<AssignmentView> {
    crate::assignments_bridge::create_assignment(&token, input).await
}

/// `/api/assignments/update` — instructor/admin edits an assignment.
#[post("/api/assignments/update")]
pub async fn assignments_update(
    token: String,
    assignment_id: uuid::Uuid,
    input: AssignmentInput,
) -> anyhow::Result<AssignmentView> {
    crate::assignments_bridge::update_assignment(&token, assignment_id, input).await
}

/// `/api/assignments/delete` — instructor/admin deletes an assignment.
#[post("/api/assignments/delete")]
pub async fn assignments_delete(token: String, assignment_id: uuid::Uuid) -> anyhow::Result<()> {
    crate::assignments_bridge::delete_assignment(&token, assignment_id).await
}

/// `/api/assignments/attach` — instructor/admin attaches a resource file.
#[post("/api/assignments/attach")]
pub async fn assignments_attach(
    token: String,
    assignment_id: uuid::Uuid,
    name: String,
    mime_type: String,
    content_base64: String,
) -> anyhow::Result<ResourceView> {
    crate::assignments_bridge::attach_resource(&token, assignment_id, &name, &mime_type, &content_base64)
        .await
}
/// Delete an existing user account (admin only).
#[post("/api/users/delete")]
pub async fn users_delete(token: String, user_id: String) -> anyhow::Result<()> {
    crate::auth_bridge::delete_user(&token, &user_id).await
}

/// Create a new user account (admin only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserInput {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub roles: Vec<String>,
}

#[post("/api/users/create")]
pub async fn users_create(token: String, input: CreateUserInput) -> anyhow::Result<AuthUser> {
    let user = crate::auth_bridge::create_user(
        &token,
        &input.username,
        &input.email,
        &input.password,
        &input.full_name,
        input.roles,
    )
    .await?;
    Ok(to_auth_user(user))
}

/// List all users (admin only).
#[post("/api/users/list")]
pub async fn users_list(token: String) -> anyhow::Result<Vec<AuthUser>> {
    let users = crate::auth_bridge::list_users(&token).await?;
    Ok(users.into_iter().map(to_auth_user).collect())
}

/// Bulk create user accounts (admin only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkCreateUserInput {
    pub users: Vec<CreateUserInput>,
}

#[post("/api/users/bulk_create")]
pub async fn users_bulk_create(token: String, input: BulkCreateUserInput) -> anyhow::Result<Vec<AuthUser>> {
    let bridge_users: Vec<crate::auth_bridge::CreateUserInput> = input
        .users
        .into_iter()
        .map(|u| crate::auth_bridge::CreateUserInput {
            username: u.username,
            email: u.email,
            password: u.password,
            full_name: u.full_name,
            roles: u.roles,
            admin_token: None,
        })
        .collect();
    let users = crate::auth_bridge::bulk_create_users(&token, bridge_users).await?;
    Ok(users.into_iter().map(to_auth_user).collect())
}

/// List available roles.
#[get("/api/auth/roles")]
pub async fn auth_roles() -> anyhow::Result<Vec<String>> {
    Ok(crate::auth_bridge::ACCOUNT_TYPES.iter().map(|s| s.to_string()).collect())
}

/// `/api/assignments/resource/download` — download an attached resource.
#[post("/api/assignments/resource/download")]
pub async fn assignments_resource_download(
    token: String,
    resource_id: uuid::Uuid,
) -> anyhow::Result<DownloadView> {
    crate::assignments_bridge::download_resource(&token, resource_id).await
}

/// `/api/assignments/submit` — student submits (or resubmits) work.
#[post("/api/assignments/submit")]
pub async fn assignments_submit(
    token: String,
    assignment_id: uuid::Uuid,
    text: Option<String>,
    files: Vec<FileUploadInput>,
    links: Vec<LinkInput>,
) -> anyhow::Result<SubmissionView> {
    crate::assignments_bridge::submit(&token, assignment_id, text, files, links).await
}

/// `/api/assignments/status` — the caller's latest submission (student) or all
/// submissions for the assignment (instructor/admin).
#[post("/api/assignments/status")]
pub async fn assignments_status(
    token: String,
    assignment_id: uuid::Uuid,
) -> anyhow::Result<serde_json::Value> {
    crate::assignments_bridge::submission_status(&token, assignment_id).await
}

/// `/api/assignments/grade` — instructor/admin grades a submission.
#[post("/api/assignments/grade")]
pub async fn assignments_grade(
    token: String,
    submission_id: uuid::Uuid,
    marks: f64,
    comments: String,
    feedback: String,
) -> anyhow::Result<GradeView> {
    crate::assignments_bridge::grade(&token, submission_id, marks, &comments, &feedback).await
}

/// `/api/assignments/history` — grade history for a submission.
#[post("/api/assignments/history")]
pub async fn assignments_history(
    token: String,
    submission_id: uuid::Uuid,
) -> anyhow::Result<GradeHistoryView> {
    crate::assignments_bridge::grade_history(&token, submission_id).await
}

/// `/api/assignments/grade/get` — a submission's current grade.
#[post("/api/assignments/grade/get")]
pub async fn assignments_grade_get(
    token: String,
    submission_id: uuid::Uuid,
) -> anyhow::Result<GradeView> {
    crate::assignments_bridge::get_grade(&token, submission_id).await
}


// ---------------------------------------------------------------------------
// Enrollments (backed by the enrollments plugin over the Redis messaging layer)
// ---------------------------------------------------------------------------

/// `/api/enrollments/self` — student self-enrolls (lands in `pending`).
#[post("/api/enrollments/self")]
pub async fn enrollments_self(
    token: String,
    course_id: uuid::Uuid,
) -> anyhow::Result<EnrollmentView> {
    crate::enrollment_bridge::self_enroll(&token, course_id).await
}

/// `/api/enrollments/enroll` — instructor/admin enrolls a student.
#[post("/api/enrollments/enroll")]
pub async fn enrollments_enroll(
    token: String,
    course_id: uuid::Uuid,
    student: String,
    expires_at: Option<String>,
) -> anyhow::Result<EnrollmentView> {
    crate::enrollment_bridge::enroll_student(&token, course_id, &student, expires_at).await
}

/// `/api/enrollments/bulk` — instructor/admin bulk-enrolls many students.
#[post("/api/enrollments/bulk")]
pub async fn enrollments_bulk(
    token: String,
    course_id: uuid::Uuid,
    students: Vec<String>,
    expires_at: Option<String>,
) -> anyhow::Result<BulkEnrollView> {
    crate::enrollment_bridge::bulk_enroll(&token, course_id, students, expires_at).await
}

/// `/api/enrollments/action` — a single state-change action
/// (approve / suspend / cancel / complete / reenroll).
#[post("/api/enrollments/action")]
pub async fn enrollments_action(
    token: String,
    action: String,
    enrollment_id: uuid::Uuid,
    reason: Option<String>,
) -> anyhow::Result<EnrollmentView> {
    crate::enrollment_bridge::change_state(&token, &action, enrollment_id, reason).await
}

/// `/api/enrollments/roster` — a course's enrollments (managers).
#[post("/api/enrollments/roster")]
pub async fn enrollments_roster(
    token: String,
    course_id: uuid::Uuid,
    state: Option<String>,
) -> anyhow::Result<Vec<EnrollmentView>> {
    crate::enrollment_bridge::roster(&token, course_id, state).await
}

/// `/api/enrollments/mine` — the caller's own enrollments.
#[post("/api/enrollments/mine")]
pub async fn enrollments_mine(token: String) -> anyhow::Result<Vec<EnrollmentView>> {
    crate::enrollment_bridge::my_enrollments(&token).await
}

/// `/api/enrollments/history` — one enrollment's audit trail.
#[post("/api/enrollments/history")]
pub async fn enrollments_history(
    token: String,
    enrollment_id: uuid::Uuid,
) -> anyhow::Result<Vec<HistoryView>> {
    crate::enrollment_bridge::history(&token, enrollment_id).await
}

/// `/api/enrollments/access` — may the caller access this course's content?
#[post("/api/enrollments/access")]
pub async fn enrollments_access(
    token: String,
    course_id: uuid::Uuid,
) -> anyhow::Result<AccessView> {
    crate::enrollment_bridge::access(&token, course_id).await
}

/// `/api/enrollments/invite/create` — manager creates a course invite link.
#[post("/api/enrollments/invite/create")]
pub async fn enrollments_invite_create(
    token: String,
    course_id: uuid::Uuid,
    expires_at: Option<String>,
    max_uses: Option<i64>,
) -> anyhow::Result<InviteView> {
    crate::enrollment_bridge::create_invite(&token, course_id, expires_at, max_uses).await
}

/// `/api/enrollments/invite/list` — a course's invites (managers).
#[post("/api/enrollments/invite/list")]
pub async fn enrollments_invite_list(
    token: String,
    course_id: uuid::Uuid,
) -> anyhow::Result<Vec<InviteView>> {
    crate::enrollment_bridge::list_invites(&token, course_id).await
}

/// `/api/enrollments/invite/revoke` — revoke an invite link.
#[post("/api/enrollments/invite/revoke")]
pub async fn enrollments_invite_revoke(
    token: String,
    invite_id: uuid::Uuid,
) -> anyhow::Result<InviteView> {
    crate::enrollment_bridge::revoke_invite(&token, invite_id).await
}

/// `/api/enrollments/join` — student joins via an invite (course link).
#[post("/api/enrollments/join")]
pub async fn enrollments_join(token: String, invite_token: String) -> anyhow::Result<EnrollmentView> {
    crate::enrollment_bridge::join(&token, &invite_token).await
}

// ---------------------------------------------------------------------------
// Courses (backed by the course_management plugin over the Redis messaging layer)
// ---------------------------------------------------------------------------

/// `/api/courses/list` — courses the caller may see.
#[post("/api/courses/list")]
pub async fn courses_list(
    token: String,
    status: Option<String>,
    mine: bool,
    search: Option<String>,
) -> anyhow::Result<Vec<CourseView>> {
    crate::course_bridge::list_courses(&token, status, mine, search).await
}

/// `/api/courses/get` — a single course.
#[post("/api/courses/get")]
pub async fn courses_get(token: String, course_id: uuid::Uuid) -> anyhow::Result<CourseView> {
    crate::course_bridge::get_course(&token, course_id).await
}

// ---------------------------------------------------------------------------
// Live classes (backed by the live_classes plugin over the Redis messaging
// layer). Real-time video/chat/whiteboard traffic goes over the separate
// `/ws/live/{session_id}` WebSocket (see `live_signaling`), not these RPCs.
// ---------------------------------------------------------------------------

/// `/api/live/list` — a course's sessions (upcoming, live and past).
#[post("/api/live/list")]
pub async fn live_list(token: String, course_id: uuid::Uuid) -> anyhow::Result<Vec<LiveSessionView>> {
    crate::live_bridge::list_for_course(&token, course_id).await
}

/// `/api/live/get` — a single session's details.
#[post("/api/live/get")]
pub async fn live_get(token: String, session_id: uuid::Uuid) -> anyhow::Result<LiveSessionView> {
    crate::live_bridge::get(&token, session_id).await
}

/// `/api/live/schedule` — instructor/admin schedules a new session.
#[post("/api/live/schedule")]
pub async fn live_schedule(
    token: String,
    course_id: uuid::Uuid,
    title: String,
    description: String,
    scheduled_start: String,
    scheduled_end: String,
    provider: String,
    external_url: Option<String>,
) -> anyhow::Result<LiveSessionView> {
    crate::live_bridge::schedule(
        &token,
        course_id,
        &title,
        &description,
        scheduled_start,
        scheduled_end,
        &provider,
        external_url,
    )
    .await
}

/// `/api/live/join` — validates access, records attendance, returns the
/// signaling path to connect to for in-app (webrtc) sessions.
#[post("/api/live/join")]
pub async fn live_join(token: String, session_id: uuid::Uuid) -> anyhow::Result<LiveJoinView> {
    crate::live_bridge::join(&token, session_id).await
}

/// `/api/live/leave` — stamp the caller's attendance as left.
#[post("/api/live/leave")]
pub async fn live_leave(token: String, attendance_id: uuid::Uuid) -> anyhow::Result<()> {
    crate::live_bridge::leave(&token, attendance_id).await
}

/// `/api/live/start` — host/instructor/admin starts a scheduled session.
#[post("/api/live/start")]
pub async fn live_start(token: String, session_id: uuid::Uuid) -> anyhow::Result<LiveSessionView> {
    crate::live_bridge::start(&token, session_id).await
}

/// `/api/live/end` — ends a live session.
#[post("/api/live/end")]
pub async fn live_end(token: String, session_id: uuid::Uuid) -> anyhow::Result<LiveSessionView> {
    crate::live_bridge::end(&token, session_id).await
}

/// `/api/live/cancel` — cancels a scheduled session.
#[post("/api/live/cancel")]
pub async fn live_cancel(token: String, session_id: uuid::Uuid) -> anyhow::Result<LiveSessionView> {
    crate::live_bridge::cancel(&token, session_id).await
}

/// `/api/live/attendance` — attendance roster for a session (managers).
#[post("/api/live/attendance")]
pub async fn live_attendance(
    token: String,
    session_id: uuid::Uuid,
) -> anyhow::Result<Vec<LiveAttendanceView>> {
    crate::live_bridge::attendance(&token, session_id).await
}

/// `/api/live/chat/history` — persisted chat history for a session.
#[post("/api/live/chat/history")]
pub async fn live_chat_history(
    token: String,
    session_id: uuid::Uuid,
) -> anyhow::Result<Vec<LiveChatMessageView>> {
    crate::live_bridge::chat_history(&token, session_id).await
}

/// `/api/live/chat/send` — send + persist a chat message (real-time relay
/// happens separately over the `/ws/live/{session_id}` WebSocket).
#[post("/api/live/chat/send")]
pub async fn live_chat_send(
    token: String,
    session_id: uuid::Uuid,
    body: String,
) -> anyhow::Result<LiveChatMessageView> {
    crate::live_bridge::send_chat(&token, session_id, &body).await
}

/// `/api/live/resource/attach` — attach a resource (link, whiteboard
/// snapshot, recording pointer) to a session.
#[post("/api/live/resource/attach")]
pub async fn live_resource_attach(
    token: String,
    session_id: uuid::Uuid,
    name: String,
    url: String,
    kind: String,
) -> anyhow::Result<LiveResourceView> {
    crate::live_bridge::attach_resource(&token, session_id, &name, &url, &kind).await
}

/// `/api/live/resource/list` — a session's attached resources.
#[post("/api/live/resource/list")]
pub async fn live_resource_list(
    token: String,
    session_id: uuid::Uuid,
) -> anyhow::Result<Vec<LiveResourceView>> {
    crate::live_bridge::list_resources(&token, session_id).await
}

/// `/api/courses/create` — instructor/admin creates a course (starts as draft).
#[post("/api/courses/create")]
pub async fn courses_create(token: String, input: CourseInput) -> anyhow::Result<CourseView> {
    crate::course_bridge::create_course(&token, input).await
}

/// `/api/courses/update` — owner/instructor edits a course.
#[post("/api/courses/update")]
pub async fn courses_update(
    token: String,
    course_id: uuid::Uuid,
    input: CourseInput,
) -> anyhow::Result<CourseView> {
    crate::course_bridge::update_course(&token, course_id, input).await
}

/// `/api/courses/delete` — owner/admin hard-deletes a course.
#[post("/api/courses/delete")]
pub async fn courses_delete(token: String, course_id: uuid::Uuid) -> anyhow::Result<()> {
    crate::course_bridge::delete_course(&token, course_id).await
}

/// `/api/courses/archive` — soft-delete (status → archived).
#[post("/api/courses/archive")]
pub async fn courses_archive(token: String, course_id: uuid::Uuid) -> anyhow::Result<CourseView> {
    crate::course_bridge::archive_course(&token, course_id).await
}

/// `/api/courses/duplicate` — copy a course as a fresh draft.
#[post("/api/courses/duplicate")]
pub async fn courses_duplicate(token: String, course_id: uuid::Uuid) -> anyhow::Result<CourseView> {
    crate::course_bridge::duplicate_course(&token, course_id).await
}

/// `/api/courses/preview` — manager-only preview regardless of status.
#[post("/api/courses/preview")]
pub async fn courses_preview(token: String, course_id: uuid::Uuid) -> anyhow::Result<CourseView> {
    crate::course_bridge::preview_course(&token, course_id).await
}

/// `/api/courses/status` — set a course's status (validates transitions).
#[post("/api/courses/status")]
pub async fn courses_set_status(
    token: String,
    course_id: uuid::Uuid,
    status: String,
) -> anyhow::Result<CourseView> {
    crate::course_bridge::set_status(&token, course_id, &status).await
}

/// `/api/courses/publish` — publish a course.
#[post("/api/courses/publish")]
pub async fn courses_publish(token: String, course_id: uuid::Uuid) -> anyhow::Result<CourseView> {
    crate::course_bridge::publish_course(&token, course_id).await
}

/// `/api/courses/unpublish` — take a course down.
#[post("/api/courses/unpublish")]
pub async fn courses_unpublish(token: String, course_id: uuid::Uuid) -> anyhow::Result<CourseView> {
    crate::course_bridge::unpublish_course(&token, course_id).await
}

// ---------------------------------------------------------------------------
// Learning Content (backed by the content plugin over the Redis messaging layer)
// ---------------------------------------------------------------------------

/// `/api/content/modules/list` — modules for a course.
#[post("/api/content/modules/list")]
pub async fn content_modules_list(
    token: String,
    course_id: uuid::Uuid,
) -> anyhow::Result<Vec<ContentModuleView>> {
    crate::content_bridge::list_modules(&token, course_id).await
}

/// `/api/content/modules/get` — a single module.
#[post("/api/content/modules/get")]
pub async fn content_modules_get(
    token: String,
    module_id: uuid::Uuid,
) -> anyhow::Result<ContentModuleView> {
    crate::content_bridge::get_module(&token, module_id).await
}

/// `/api/content/modules/create` — create a module.
#[post("/api/content/modules/create")]
pub async fn content_modules_create(
    token: String,
    input: CreateModuleInput,
) -> anyhow::Result<ContentModuleView> {
    crate::content_bridge::create_module(&token, input).await
}

/// `/api/content/modules/update` — update a module.
#[post("/api/content/modules/update")]
pub async fn content_modules_update(
    token: String,
    input: UpdateModuleInput,
) -> anyhow::Result<ContentModuleView> {
    crate::content_bridge::update_module(&token, input).await
}

/// `/api/content/modules/delete` — delete a module.
#[post("/api/content/modules/delete")]
pub async fn content_modules_delete(
    token: String,
    module_id: uuid::Uuid,
) -> anyhow::Result<()> {
    crate::content_bridge::delete_module(&token, module_id).await
}

/// `/api/content/modules/reorder` — reorder modules within a course.
#[post("/api/content/modules/reorder")]
pub async fn content_modules_reorder(
    token: String,
    course_id: uuid::Uuid,
    module_ids: Vec<uuid::Uuid>,
) -> anyhow::Result<()> {
    crate::content_bridge::reorder_modules(&token, course_id, module_ids).await
}

/// `/api/content/modules/status` — publish/unpublish a module.
#[post("/api/content/modules/status")]
pub async fn content_modules_status(
    token: String,
    module_id: uuid::Uuid,
    status: String,
) -> anyhow::Result<ContentModuleView> {
    crate::content_bridge::set_module_status(&token, module_id, &status).await
}

/// `/api/content/lessons/list` — lessons for a module.
#[post("/api/content/lessons/list")]
pub async fn content_lessons_list(
    token: String,
    module_id: uuid::Uuid,
) -> anyhow::Result<Vec<ContentLessonView>> {
    crate::content_bridge::list_lessons(&token, module_id).await
}

/// `/api/content/lessons/get` — a single lesson.
#[post("/api/content/lessons/get")]
pub async fn content_lessons_get(
    token: String,
    lesson_id: uuid::Uuid,
) -> anyhow::Result<ContentLessonView> {
    crate::content_bridge::get_lesson(&token, lesson_id).await
}

/// `/api/content/lessons/create` — create a lesson.
#[post("/api/content/lessons/create")]
pub async fn content_lessons_create(
    token: String,
    input: CreateLessonInput,
) -> anyhow::Result<ContentLessonView> {
    crate::content_bridge::create_lesson(&token, input).await
}

/// `/api/content/lessons/update` — update a lesson.
#[post("/api/content/lessons/update")]
pub async fn content_lessons_update(
    token: String,
    input: UpdateLessonInput,
) -> anyhow::Result<ContentLessonView> {
    crate::content_bridge::update_lesson(&token, input).await
}

/// `/api/content/lessons/delete` — delete a lesson.
#[post("/api/content/lessons/delete")]
pub async fn content_lessons_delete(
    token: String,
    lesson_id: uuid::Uuid,
) -> anyhow::Result<()> {
    crate::content_bridge::delete_lesson(&token, lesson_id).await
}

/// `/api/content/lessons/reorder` — reorder lessons within a module.
#[post("/api/content/lessons/reorder")]
pub async fn content_lessons_reorder(
    token: String,
    module_id: uuid::Uuid,
    lesson_ids: Vec<uuid::Uuid>,
) -> anyhow::Result<()> {
    crate::content_bridge::reorder_lessons(&token, module_id, lesson_ids).await
}

/// `/api/content/lessons/status` — publish/unpublish a lesson.
#[post("/api/content/lessons/status")]
pub async fn content_lessons_status(
    token: String,
    lesson_id: uuid::Uuid,
    status: String,
) -> anyhow::Result<ContentLessonView> {
    crate::content_bridge::set_lesson_status(&token, lesson_id, &status).await
}

/// `/api/content/items/list` — content items for a lesson.
#[post("/api/content/items/list")]
pub async fn content_items_list(
    token: String,
    lesson_id: uuid::Uuid,
) -> anyhow::Result<Vec<ContentItemView>> {
    crate::content_bridge::list_items(&token, lesson_id).await
}

/// `/api/content/items/get` — a single content item.
#[post("/api/content/items/get")]
pub async fn content_items_get(
    token: String,
    item_id: uuid::Uuid,
) -> anyhow::Result<ContentItemView> {
    crate::content_bridge::get_item(&token, item_id).await
}

/// `/api/content/items/create` — create a content item.
#[post("/api/content/items/create")]
pub async fn content_items_create(
    token: String,
    input: CreateContentItemInput,
) -> anyhow::Result<ContentItemView> {
    crate::content_bridge::create_item(&token, input).await
}

/// `/api/content/items/update` — update a content item.
#[post("/api/content/items/update")]
pub async fn content_items_update(
    token: String,
    input: UpdateContentItemInput,
) -> anyhow::Result<ContentItemView> {
    crate::content_bridge::update_item(&token, input).await
}

/// `/api/content/items/delete` — delete a content item.
#[post("/api/content/items/delete")]
pub async fn content_items_delete(
    token: String,
    item_id: uuid::Uuid,
) -> anyhow::Result<()> {
    crate::content_bridge::delete_item(&token, item_id).await
}

/// `/api/content/items/reorder` — reorder content items within a lesson.
#[post("/api/content/items/reorder")]
pub async fn content_items_reorder(
    token: String,
    lesson_id: uuid::Uuid,
    item_ids: Vec<uuid::Uuid>,
) -> anyhow::Result<()> {
    crate::content_bridge::reorder_items(&token, lesson_id, item_ids).await
}

/// `/api/content/files/upload` — upload a file attachment.
#[post("/api/content/files/upload")]
pub async fn content_files_upload(
    token: String,
    course_id: uuid::Uuid,
    filename: String,
    mime_type: String,
    size_bytes: i64,
    content_base64: String,
) -> anyhow::Result<ContentFileView> {
    crate::content_bridge::upload_file(&token, course_id, &filename, &mime_type, size_bytes, &content_base64).await
}

/// `/api/content/files/list` — files for a course.
#[post("/api/content/files/list")]
pub async fn content_files_list(
    token: String,
    course_id: uuid::Uuid,
) -> anyhow::Result<Vec<ContentFileView>> {
    crate::content_bridge::list_files(&token, course_id).await
}

/// `/api/content/files/get` — a single file record.
#[post("/api/content/files/get")]
pub async fn content_files_get(
    token: String,
    file_id: uuid::Uuid,
) -> anyhow::Result<ContentFileView> {
    crate::content_bridge::get_file(&token, file_id).await
}

/// `/api/content/files/download` — download a file (base64-encoded, authenticated).
#[post("/api/content/files/download")]
pub async fn content_files_download(
    token: String,
    file_id: uuid::Uuid,
) -> anyhow::Result<FileDownloadView> {
    crate::content_bridge::download_file(&token, file_id).await
}

/// `/api/content/files/delete` — delete a file attachment.
#[post("/api/content/files/delete")]
pub async fn content_files_delete(
    token: String,
    file_id: uuid::Uuid,
) -> anyhow::Result<()> {
    crate::content_bridge::delete_file(&token, file_id).await
}

