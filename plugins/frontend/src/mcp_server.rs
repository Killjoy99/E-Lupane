//! Model Context Protocol (MCP) endpoint — **Streamable HTTP** transport.
//!
//! This exposes E-Lupane's notification operations to external AI clients
//! (Claude Desktop, Cline, VS Code, …) as MCP *tools*, so an agent can read and
//! act on a signed-in user's notifications instead of a human clicking the bell
//! in `dashboard.rs`.
//!
//! ```text
//! AI client ──POST /api/mcp──▶ frontend Axum server ──Redis stream──▶ notifications plugin
//!  (JSON-RPC 2.0,               (this module:                    │
//!   Bearer <session JWT>)        verifies JWT, maps tool→op)     └── reply channel ──┘
//! ```
//!
//! Design notes (mirrors the other frontend bridges):
//!
//! * **Reuses the frontend session JWT.** The caller's `Authorization: Bearer
//!   <token>` is the same auth-plugin JWT the browser holds, verified through
//!   [`crate::auth_bridge::verify`] — no separate credential, and no token is
//!   minted here.
//! * **The caller can never choose whose notifications to touch.** Every read /
//!   mark tool injects the *verified* user id into the plugin payload, so the
//!   model cannot ask for another user's data (the plugin's ops only validate
//!   the JWT, not ownership — see `plugins/notifications/src/handlers.rs`).
//! * **The transport is stateless here**: a single `POST` carries one (or a
//!   batch of) JSON-RPC 2.0 messages and the reply is plain JSON. Session ids
//!   are echoed for clients that expect them, but no server-side session state
//!   is kept and `GET` (server-initiated SSE) is answered with 405, which the
//!   spec permits.
//!
//! Only the notifications plugin is mapped today; adding another plugin means
//! appending to [`TOOLS`], extending the payload mapping in [`build_payload`],
//! and adding a schema function.

use std::time::Duration;

use anyhow::{anyhow, Result};
use dioxus::server::axum::extract::Query;
use dioxus::server::axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use dioxus::server::axum::response::{IntoResponse, Response};
use dioxus::server::axum::Json;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use messaging_core::rpc::RequestClient;

/// The notifications plugin's registered id (`plugins/notifications/plugin.toml`).
pub const NOTIFICATIONS_PLUGIN: &str = "com.elupane.notifications";

/// Who we are on the messaging layer (appears as `Message::source`).
const SOURCE: &str = "com.elupane.frontend.mcp";

/// The MCP revision this server implements.
const LATEST_PROTOCOL: &str = "2025-06-18";

/// Revisions we can speak when a client asks for an older one.
const SUPPORTED_PROTOCOLS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

/// The path clients are configured with.
pub const MCP_PATH: &str = "/api/mcp";

/// Plugin RPC timeout (same budget as the other frontend bridges).
const RPC_TIMEOUT: Duration = Duration::from_secs(10);

/// Roles the notifications plugin accepts for `notifications.send`
/// (its `authz::INSTRUCTOR_ROLES`).
const INSTRUCTOR_ROLES: &[&str] = &["admin", "lecturer"];
// ---------------------------------------------------------------------------
// Generic plugin-op catalog (every plugin, every operation)
// ---------------------------------------------------------------------------

/// Plugin ids used by the generic tool catalog.
const ACADEMICS_PLUGIN: &str = "com.elupane.academics";
const ANALYTICS_PLUGIN: &str = "com.elupane.analytics";
const ASSIGNMENTS_PLUGIN: &str = "com.elupane.assignments";
const AUTH_PLUGIN: &str = "com.elupane.auth";
const CERTIFICATES_PLUGIN: &str = "com.elupane.certificates";
const CONTENT_PLUGIN: &str = "com.elupane.content";
const COURSE_MANAGEMENT_PLUGIN: &str = "com.elupane.course_management";
const ENROLLMENTS_PLUGIN: &str = "com.elupane.enrollments";
const GRADES_PLUGIN: &str = "com.elupane.grades";
const INSTITUTION_MANAGEMENT_PLUGIN: &str = "com.elupane.institution_management";
const LEARNING_PROGRESS_PLUGIN: &str = "com.elupane.learning_progress";
const LIVE_CLASSES_PLUGIN: &str = "com.elupane.live_classes";
const MEDIA_PLUGIN: &str = "com.elupane.media_service";
const MESSAGING_PLUGIN: &str = "com.elupane.messaging";
const QUIZZES_PLUGIN: &str = "com.elupane.quizzes";
const SEARCH_PLUGIN: &str = "com.elupane.search";
const SUPPORT_PLUGIN: &str = "com.elupane.support";

/// One generic tool per plugin operation (notifications is excluded — it has
/// hand-written typed tools — and `events.subscribe` internals are not
/// AI-callable). Entries are `(plugin id, short name, op)`.
///
/// Tool names are derived by [`tool_name`]; arguments are passed through to the
/// plugin op as a JSON object (see [`build_generic_payload`]), with the verified
/// session JWT injected. Role/ownership enforcement stays inside each plugin.
const GENERIC_TOOLS: &[(&str, &str, &str)] = &[
    // ---- academics ----
    (ACADEMICS_PLUGIN, "academics", "faculty.create"),
    (ACADEMICS_PLUGIN, "academics", "faculty.get"),
    (ACADEMICS_PLUGIN, "academics", "faculty.list"),
    (ACADEMICS_PLUGIN, "academics", "faculty.update"),
    (ACADEMICS_PLUGIN, "academics", "faculty.delete"),
    (ACADEMICS_PLUGIN, "academics", "department.create"),
    (ACADEMICS_PLUGIN, "academics", "department.get"),
    (ACADEMICS_PLUGIN, "academics", "department.list"),
    (ACADEMICS_PLUGIN, "academics", "department.update"),
    (ACADEMICS_PLUGIN, "academics", "department.delete"),
    (ACADEMICS_PLUGIN, "academics", "program.create"),
    (ACADEMICS_PLUGIN, "academics", "program.get"),
    (ACADEMICS_PLUGIN, "academics", "program.list"),
    (ACADEMICS_PLUGIN, "academics", "program.update"),
    (ACADEMICS_PLUGIN, "academics", "program.delete"),
    (ACADEMICS_PLUGIN, "academics", "year.create"),
    (ACADEMICS_PLUGIN, "academics", "year.get"),
    (ACADEMICS_PLUGIN, "academics", "year.list"),
    (ACADEMICS_PLUGIN, "academics", "year.update"),
    (ACADEMICS_PLUGIN, "academics", "year.delete"),
    (ACADEMICS_PLUGIN, "academics", "semester.create"),
    (ACADEMICS_PLUGIN, "academics", "semester.get"),
    (ACADEMICS_PLUGIN, "academics", "semester.list"),
    (ACADEMICS_PLUGIN, "academics", "semester.update"),
    (ACADEMICS_PLUGIN, "academics", "semester.delete"),
    (ACADEMICS_PLUGIN, "academics", "class.create"),
    (ACADEMICS_PLUGIN, "academics", "class.get"),
    (ACADEMICS_PLUGIN, "academics", "class.list"),
    (ACADEMICS_PLUGIN, "academics", "class.update"),
    (ACADEMICS_PLUGIN, "academics", "class.delete"),
    (ACADEMICS_PLUGIN, "academics", "offering.create"),
    (ACADEMICS_PLUGIN, "academics", "offering.get"),
    (ACADEMICS_PLUGIN, "academics", "offering.list"),
    (ACADEMICS_PLUGIN, "academics", "offering.update"),
    (ACADEMICS_PLUGIN, "academics", "offering.delete"),

    // ---- analytics ----
    (ANALYTICS_PLUGIN, "analytics", "analytics.student.progress"),
    (ANALYTICS_PLUGIN, "analytics", "analytics.instructor.overview"),
    (ANALYTICS_PLUGIN, "analytics", "analytics.admin.overview"),
    (ANALYTICS_PLUGIN, "analytics", "analytics.reports.generate"),
    (ANALYTICS_PLUGIN, "analytics", "analytics.reports.list"),
    // ---- assignments ----
    (ASSIGNMENTS_PLUGIN, "assignments", "assignment.create"),
    (ASSIGNMENTS_PLUGIN, "assignments", "assignment.update"),
    (ASSIGNMENTS_PLUGIN, "assignments", "assignment.delete"),
    (ASSIGNMENTS_PLUGIN, "assignments", "assignment.get"),
    (ASSIGNMENTS_PLUGIN, "assignments", "assignment.list"),
    (ASSIGNMENTS_PLUGIN, "assignments", "assignment.resource.attach"),
    (ASSIGNMENTS_PLUGIN, "assignments", "assignment.resource.remove"),
    (ASSIGNMENTS_PLUGIN, "assignments", "assignment.resource.download"),
    (ASSIGNMENTS_PLUGIN, "assignments", "submission.submit"),
    (ASSIGNMENTS_PLUGIN, "assignments", "submission.status"),
    (ASSIGNMENTS_PLUGIN, "assignments", "submission.download"),
    (ASSIGNMENTS_PLUGIN, "assignments", "grade.create"),
    (ASSIGNMENTS_PLUGIN, "assignments", "grade.get"),
    (ASSIGNMENTS_PLUGIN, "assignments", "grade.history"),
    (ASSIGNMENTS_PLUGIN, "assignments", "rubric.create"),
    (ASSIGNMENTS_PLUGIN, "assignments", "rubric.list"),
    (ASSIGNMENTS_PLUGIN, "assignments", "rubric.get"),
    // ---- auth ----
    (AUTH_PLUGIN, "auth", "user.create"),
    (AUTH_PLUGIN, "auth", "user.bulk_create"),
    (AUTH_PLUGIN, "auth", "user.get"),
    (AUTH_PLUGIN, "auth", "user.list"),
    (AUTH_PLUGIN, "auth", "user.delete"),
    (AUTH_PLUGIN, "auth", "auth.login"),
    (AUTH_PLUGIN, "auth", "auth.google_login"),
    (AUTH_PLUGIN, "auth", "auth.verify"),
    (AUTH_PLUGIN, "auth", "role.assign"),
    (AUTH_PLUGIN, "auth", "role.list"),
    (AUTH_PLUGIN, "auth", "permission.list"),
    (AUTH_PLUGIN, "auth", "role.revoke"),
    (AUTH_PLUGIN, "auth", "role.permissions.set"),
    (AUTH_PLUGIN, "auth", "role.create"),
    // ---- certificates ----
    (CERTIFICATES_PLUGIN, "certificates", "certificate.generate"),
    (CERTIFICATES_PLUGIN, "certificates", "certificate.list"),
    (CERTIFICATES_PLUGIN, "certificates", "certificate.get"),
    (CERTIFICATES_PLUGIN, "certificates", "certificate.verify"),

    // ---- content ----
    (CONTENT_PLUGIN, "content", "content.module.list"),
    (CONTENT_PLUGIN, "content", "content.module.get"),
    (CONTENT_PLUGIN, "content", "content.module.create"),
    (CONTENT_PLUGIN, "content", "content.module.update"),
    (CONTENT_PLUGIN, "content", "content.module.delete"),
    (CONTENT_PLUGIN, "content", "content.module.reorder"),
    (CONTENT_PLUGIN, "content", "content.module.set_status"),
    (CONTENT_PLUGIN, "content", "content.lesson.list"),
    (CONTENT_PLUGIN, "content", "content.lesson.get"),
    (CONTENT_PLUGIN, "content", "content.lesson.create"),
    (CONTENT_PLUGIN, "content", "content.lesson.update"),
    (CONTENT_PLUGIN, "content", "content.lesson.delete"),
    (CONTENT_PLUGIN, "content", "content.lesson.reorder"),
    (CONTENT_PLUGIN, "content", "content.lesson.set_status"),
    (CONTENT_PLUGIN, "content", "content.item.list"),
    (CONTENT_PLUGIN, "content", "content.item.get"),
    (CONTENT_PLUGIN, "content", "content.item.create"),
    (CONTENT_PLUGIN, "content", "content.item.update"),
    (CONTENT_PLUGIN, "content", "content.item.delete"),
    (CONTENT_PLUGIN, "content", "content.item.reorder"),
    (CONTENT_PLUGIN, "content", "content.file.upload"),
    (CONTENT_PLUGIN, "content", "content.file.list"),
    (CONTENT_PLUGIN, "content", "content.file.get"),
    (CONTENT_PLUGIN, "content", "content.file.download"),
    (CONTENT_PLUGIN, "content", "content.file.delete"),
    (CONTENT_PLUGIN, "content", "content.material.add"),
    // ---- course_management ----
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.create"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.get"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.list"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.update"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.delete"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.archive"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.duplicate"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.preview"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.set_status"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.publish"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.unpublish"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.submit_review"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.instructor.add"),
    (COURSE_MANAGEMENT_PLUGIN, "course_management", "course.instructor.remove"),
    // ---- enrollments ----
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.self"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.enroll"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.bulk"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.approve"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.suspend"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.cancel"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.complete"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.reenroll"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.roster"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.mine"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.get"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.history"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.access"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.invite.create"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.invite.list"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.invite.revoke"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.join"),
    (ENROLLMENTS_PLUGIN, "enrollments", "enrollment.sweep"),
    // ---- grades ----
    (GRADES_PLUGIN, "grades", "grades.category.list"),
    (GRADES_PLUGIN, "grades", "grades.category.create"),
    (GRADES_PLUGIN, "grades", "grades.category.update"),
    (GRADES_PLUGIN, "grades", "grades.category.delete"),
    (GRADES_PLUGIN, "grades", "grades.scale.list"),
    (GRADES_PLUGIN, "grades", "grades.scale.create"),
    (GRADES_PLUGIN, "grades", "grades.entry.create"),
    (GRADES_PLUGIN, "grades", "grades.entry.list"),
    (GRADES_PLUGIN, "grades", "grades.entry.override"),
    (GRADES_PLUGIN, "grades", "grades.entry.history"),
    (GRADES_PLUGIN, "grades", "grades.final.get"),
    (GRADES_PLUGIN, "grades", "grades.final.calculate"),
    (GRADES_PLUGIN, "grades", "grades.final.list"),
    (GRADES_PLUGIN, "grades", "grades.student.view"),
    // ---- institution_management ----
    (INSTITUTION_MANAGEMENT_PLUGIN, "institution_management", "institution.create"),
    (INSTITUTION_MANAGEMENT_PLUGIN, "institution_management", "institution.get"),
    (INSTITUTION_MANAGEMENT_PLUGIN, "institution_management", "institution.list"),
    (INSTITUTION_MANAGEMENT_PLUGIN, "institution_management", "institution.update"),
    (INSTITUTION_MANAGEMENT_PLUGIN, "institution_management", "institution.delete"),
    (INSTITUTION_MANAGEMENT_PLUGIN, "institution_management", "campus.create"),
    (INSTITUTION_MANAGEMENT_PLUGIN, "institution_management", "campus.get"),
    (INSTITUTION_MANAGEMENT_PLUGIN, "institution_management", "campus.list"),
    (INSTITUTION_MANAGEMENT_PLUGIN, "institution_management", "campus.update"),
    (INSTITUTION_MANAGEMENT_PLUGIN, "institution_management", "campus.delete"),
    // ---- learning_progress ----
    (LEARNING_PROGRESS_PLUGIN, "learning_progress", "learning_progress.get_or_create"),
    (LEARNING_PROGRESS_PLUGIN, "learning_progress", "learning_progress.update"),
    (LEARNING_PROGRESS_PLUGIN, "learning_progress", "learning_progress.mark_complete"),
    (LEARNING_PROGRESS_PLUGIN, "learning_progress", "learning_progress.course_summary"),
    (LEARNING_PROGRESS_PLUGIN, "learning_progress", "learning_progress.resume"),
    (LEARNING_PROGRESS_PLUGIN, "learning_progress", "learning_progress.events"),
    // ---- live_classes ----
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.schedule"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.update"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.list"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.get"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.join"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.start"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.end"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.cancel"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.leave"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.attendance.list"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.chat.send"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.chat.history"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.resource.attach"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.resource.list"),
    (LIVE_CLASSES_PLUGIN, "live_classes", "live.recording.save"),
    // ---- media ----
    (MEDIA_PLUGIN, "media", "media.upload.request"),
    (MEDIA_PLUGIN, "media", "media.upload.complete"),
    (MEDIA_PLUGIN, "media", "media.download.request"),
    (MEDIA_PLUGIN, "media", "media.delete"),
    (MEDIA_PLUGIN, "media", "media.list"),
    (MEDIA_PLUGIN, "media", "media.get"),
    // ---- messaging ----
    (MESSAGING_PLUGIN, "messaging", "messaging.discussion.list"),
    (MESSAGING_PLUGIN, "messaging", "messaging.discussion.create"),
    (MESSAGING_PLUGIN, "messaging", "messaging.discussion.update"),
    (MESSAGING_PLUGIN, "messaging", "messaging.discussion.delete"),
    (MESSAGING_PLUGIN, "messaging", "messaging.discussion.reply"),
    (MESSAGING_PLUGIN, "messaging", "messaging.discussion.list_replies"),
    (MESSAGING_PLUGIN, "messaging", "messaging.discussion.mark_answer"),
    // ---- quizzes ----
    (QUIZZES_PLUGIN, "quizzes", "quizzes.list"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.get"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.create"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.update"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.delete"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.questions.list"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.questions.create"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.attempts.start"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.attempts.submit_answer"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.attempts.submit"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.attempts.grade"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.attempts.get"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.attempts.answers"),
    (QUIZZES_PLUGIN, "quizzes", "quizzes.student.view"),
    // ---- search ----
    (SEARCH_PLUGIN, "search", "search.courses"),
    (SEARCH_PLUGIN, "search", "search.content"),
    (SEARCH_PLUGIN, "search", "search.all"),
    // ---- support ----
    (SUPPORT_PLUGIN, "support", "support.ticket.create"),
    (SUPPORT_PLUGIN, "support", "support.ticket.list"),
    (SUPPORT_PLUGIN, "support", "support.ticket.get"),
    (SUPPORT_PLUGIN, "support", "support.ticket.assign"),
    (SUPPORT_PLUGIN, "support", "support.ticket.set_status"),
    (SUPPORT_PLUGIN, "support", "support.message.create"),
    (SUPPORT_PLUGIN, "support", "support.faq.list"),
    (SUPPORT_PLUGIN, "support", "support.faq.create"),
    (SUPPORT_PLUGIN, "support", "support.faq.update"),
    (SUPPORT_PLUGIN, "support", "support.faq.delete"),
];


/// Ops whose DTO expects the caller's JWT under `admin_token`
/// (`plugins/auth/src/handlers.rs::require_admin`) instead of `token`.
fn needs_admin_token(op: &str) -> bool {
    matches!(
        op,
        "user.create"
            | "user.bulk_create"
            | "user.delete"
            | "role.assign"
            | "role.revoke"
            | "role.permissions.set"
            | "role.create"
    )
}

/// The MCP tool name for a generic catalog entry: `<short>_<op with '.' → '_'>`,
/// collapsing the redundant leading plugin name (e.g. `messaging` +
/// `messaging.discussion.list` → `messaging_discussion_list`).
fn tool_name(short: &str, op: &str) -> String {
    let dotted = op.replace('.', "_");
    if op.starts_with(short) {
        dotted
    } else {
        format!("{short}_{dotted}")
    }
}

/// Human-readable description for a generic tool.
fn tool_description(short: &str, op: &str) -> String {
    let mut description = format!(
        "Call `{op}` on the `{short}` plugin. Arguments are passed through to the \
         plugin operation as a JSON object (the plugin validates them and reports \
         which fields are missing or invalid). Authentication is enforced by the plugin."
    );
    if is_destructive(op) {
        description.push_str(
            " DESTRUCTIVE: this operation deletes, cancels or otherwise removes data — confirm with the user before calling.",
        );
    }
    description
}

/// Ops that remove or irreversibly change data (`annotations.destructiveHint`).
fn is_destructive(op: &str) -> bool {
    const DESTRUCTIVE_MARKERS: &[&str] = &[
        ".delete", ".remove", ".cancel", ".suspend", ".revoke", ".archive", ".end", ".unpublish",
    ];
    DESTRUCTIVE_MARKERS.iter().any(|marker| op.ends_with(marker))
}

/// Input schema for generic tools: a free-form JSON object. The plugin rejects
/// missing/invalid fields with a descriptive error the model can correct.
fn schema_pass_through() -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "description": "JSON payload forwarded to the plugin operation. Do not include 'token' — the signed-in session token is injected automatically."
    })
}



// ---------------------------------------------------------------------------
// Transport: authentication
// ---------------------------------------------------------------------------

/// `?token=...` fallback for clients that cannot set request headers.
#[derive(Debug, Deserialize)]
pub struct AuthParams {
    #[serde(default)]
    token: Option<String>,
}

/// The verified caller behind an MCP request.
struct Caller {
    /// The session JWT, forwarded verbatim to the plugin (its ops expect
    /// `token` *inside* the payload — see `plugins/notifications/src/handlers.rs`).
    token: String,
    user_id: Uuid,
    roles: Vec<String>,
}

impl Caller {
    fn is_instructor(&self) -> bool {
        self.roles
            .iter()
            .any(|role| INSTRUCTOR_ROLES.contains(&role.as_str()))
    }
}

/// Pull the bearer token from `Authorization`, falling back to `?token=`.
fn bearer_token(headers: &HeaderMap, params: &AuthParams) -> Option<String> {
    if let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = value
            .strip_prefix("Bearer ")
            .or_else(|| value.strip_prefix("bearer "))
        {
            let token = token.trim();
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }
    params.token.clone().filter(|token| !token.is_empty())
}

/// Verify the session JWT against the auth plugin and resolve the caller.
async fn authenticate(headers: &HeaderMap, params: &AuthParams) -> Result<Caller, Response> {
    let Some(token) = bearer_token(headers, params) else {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "missing credentials: send 'Authorization: Bearer <session token>'",
        ));
    };

    match crate::auth_bridge::verify(&token).await {
        Ok(Some(user)) => match Uuid::parse_str(&user.id) {
            Ok(user_id) => Ok(Caller {
                token,
                user_id,
                roles: user.roles,
            }),
            Err(_) => Err(error_response(
                StatusCode::UNAUTHORIZED,
                "session user id is not a uuid",
            )),
        },
        Ok(None) => Err(error_response(
            StatusCode::UNAUTHORIZED,
            "invalid or expired session token",
        )),
        // Redis or the auth plugin being unreachable is a server-side failure,
        // not a credential failure.
        Err(err) => Err(error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            &format!("auth plugin unavailable: {err}"),
        )),
    }
}

// ---------------------------------------------------------------------------
// Tool catalog
// ---------------------------------------------------------------------------

/// One MCP tool: a JSON-Schema-described wrapper around a plugin operation.
struct ToolSpec {
    /// MCP tool name (as the model sees it).
    name: &'static str,
    /// What the model is told this does, including side effects.
    description: &'static str,
    /// The plugin operation this maps onto.
    op: &'static str,
    /// The `tools/list` input schema.
    schema: fn() -> Value,
}

/// Every tool this endpoint exposes.
const TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "notifications_list",
        description: "List every in-app notification of the signed-in user, newest first. Read-only.",
        op: "notifications.list",
        schema: schema_no_args,
    },
    ToolSpec {
        name: "notifications_unread",
        description: "List the signed-in user's unread in-app notifications, newest first. Read-only.",
        op: "notifications.unread",
        schema: schema_no_args,
    },
    ToolSpec {
        name: "notifications_mark_read",
        description: "Mark one of the signed-in user's notifications as read. Mutates state.",
        op: "notifications.mark_read",
        schema: schema_notification_id,
    },
    ToolSpec {
        name: "notifications_mark_all_read",
        description: "Mark all of the signed-in user's unread notifications as read. Mutates state; returns how many were marked.",
        op: "notifications.mark_all_read",
        schema: schema_no_args,
    },
    ToolSpec {
        name: "notifications_send",
        description: "Create and deliver an in-app notification to a user. Requires the admin or lecturer role.",
        op: "notifications.send",
        schema: schema_send,
    },
    ToolSpec {
        name: "notifications_preferences_list",
        description: "List the signed-in user's per-type notification channel preferences. Read-only.",
        op: "notifications.preferences.list",
        schema: schema_no_args,
    },
    ToolSpec {
        name: "notifications_preferences_set",
        description: "Enable or disable one notification channel (in_app/email/push) for one notification type for the signed-in user. Mutates state.",
        op: "notifications.preferences.set",
        schema: schema_preferences_set,
    },
    ToolSpec {
        name: "notifications_deliveries",
        description: "List the delivery attempts (channel, status, error) recorded for one notification. Read-only.",
        op: "notifications.deliveries",
        schema: schema_deliveries,
    },
];

fn schema_no_args() -> Value {
    json!({ "type": "object", "properties": {}, "additionalProperties": false })
}

fn schema_notification_id() -> Value {
    json!({
        "type": "object",
        "properties": {
            "notification_id": {
                "type": "string",
                "format": "uuid",
                "description": "Id of the notification (as returned by notifications_list)."
            }
        },
        "required": ["notification_id"],
        "additionalProperties": false
    })
}

fn schema_send() -> Value {
    json!({
        "type": "object",
        "properties": {
            "user_id": {
                "type": "string",
                "format": "uuid",
                "description": "Recipient user id."
            },
            "notification_type": {
                "type": "string",
                "description": "Notification category stored on the row (e.g. message, announcement, support, certificate)."
            },
            "title": { "type": "string", "description": "Short notification title." },
            "body": { "type": "string", "description": "Optional notification body." },
            "data": {
                "type": "object",
                "description": "Optional structured payload stored with the notification."
            }
        },
        "required": ["user_id", "notification_type", "title"],
        "additionalProperties": false
    })
}

fn schema_preferences_set() -> Value {
    json!({
        "type": "object",
        "properties": {
            "notification_type": {
                "type": "string",
                "description": "Notification type the preference applies to."
            },
            "channel": {
                "type": "string",
                "enum": ["in_app", "email", "push"],
                "description": "Delivery channel."
            },
            "enabled": { "type": "boolean", "description": "Whether the channel is enabled." }
        },
        "required": ["notification_type", "channel", "enabled"],
        "additionalProperties": false
    })
}

fn schema_deliveries() -> Value {
    json!({
        "type": "object",
        "properties": {
            "notification_id": {
                "type": "string",
                "format": "uuid",
                "description": "Notification whose delivery log to read."
            }
        },
        "required": ["notification_id"],
        "additionalProperties": false
    })
}

// ---------------------------------------------------------------------------
// Tool arguments → plugin payloads
// ---------------------------------------------------------------------------

fn arg_str(args: &Value, key: &str) -> Result<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("missing required string argument '{key}'"))
}

fn arg_bool(args: &Value, key: &str) -> Result<bool> {
    args.get(key)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| anyhow!("missing required boolean argument '{key}'"))
}

fn arg_optional_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Translate MCP tool arguments into the payload the notifications plugin's
/// `handlers::dispatch` expects.
///
/// The verified session token is always injected, and every user-scoped op gets
/// the *caller's* id — never a value supplied by the model.
fn build_payload(spec: &ToolSpec, args: &Value, caller: &Caller) -> Result<Value> {
    let token = Value::String(caller.token.clone());
    let user_id = Value::String(caller.user_id.to_string());

    let payload = match spec.op {
        // Read/mark operations are always scoped to the signed-in user: the
        // plugin checks the JWT but not ownership, so this is what keeps one
        // user's notifications out of reach of another user's agent.
        "notifications.list"
        | "notifications.unread"
        | "notifications.mark_all_read"
        | "notifications.preferences.list" => json!({ "token": token, "user_id": user_id }),

        "notifications.mark_read" => json!({
            "token": token,
            "notification_id": arg_str(args, "notification_id")?,
        }),

        // `notifications.deliveries` names its input `id` (`dto::IdRequest`).
        "notifications.deliveries" => json!({
            "token": token,
            "id": arg_str(args, "notification_id")?,
        }),

        // `preferences.set` takes the user from the token claims, so no user_id.
        "notifications.preferences.set" => json!({
            "token": token,
            "notification_type": arg_str(args, "notification_type")?,
            "channel": arg_str(args, "channel")?,
            "enabled": arg_bool(args, "enabled")?,
        }),

        "notifications.send" => {
            if !caller.is_instructor() {
                return Err(anyhow!(
                    "forbidden: '{}' requires the admin or lecturer role",
                    spec.name
                ));
            }
            let mut payload = Map::new();
            payload.insert("token".to_string(), token);
            payload.insert(
                "user_id".to_string(),
                arg_str(args, "user_id")?.into(),
            );
            payload.insert(
                "notification_type".to_string(),
                arg_str(args, "notification_type")?.into(),
            );
            payload.insert("title".to_string(), arg_str(args, "title")?.into());
            if let Some(body) = arg_optional_str(args, "body") {
                payload.insert("body".to_string(), body.into());
            }
            if let Some(data) = args.get("data").filter(|v| v.is_object()) {
                payload.insert("data".to_string(), data.clone());
            }
            Value::Object(payload)
        }

        other => return Err(anyhow!("no payload mapping for op '{other}'")),
    };

    Ok(payload)
}

/// Translate generic tool arguments into the payload for any plugin op.
///
/// The verified session JWT is always injected — as `token`, or as
/// `admin_token` for the auth ops whose DTOs expect it there. Any model-supplied
/// credential fields are stripped first so the AI can never substitute its own.
fn build_generic_payload(op: &str, args: &Value, caller: &Caller) -> Result<Value> {
    let mut payload = match args {
        Value::Object(map) => map.clone(),
        _ => Map::new(),
    };
    payload.remove("token");
    payload.remove("admin_token");

    let credential_key = if needs_admin_token(op) {
        "admin_token"
    } else {
        "token"
    };
    payload.insert(
        credential_key.to_string(),
        Value::String(caller.token.clone()),
    );

    Ok(Value::Object(payload))
}

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 dispatch
// ---------------------------------------------------------------------------

const JSONRPC_VERSION: &str = "2.0";

fn rpc_result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": JSONRPC_VERSION, "id": id, "result": result })
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "error": { "code": code, "message": message }
    })
}

/// A `tools/call` success: the plugin's reply payload as JSON text.
fn tool_result(payload: Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "isError": false
    })
}

/// A `tools/call` failure the model should read and recover from.
fn tool_error(message: &str) -> Value {
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true
    })
}

/// The plugin ops that only read state (`annotations.readOnlyHint`).
fn is_read_only(op: &str) -> bool {
    op.ends_with(".list")
        || op.ends_with(".unread")
        || op.ends_with(".deliveries")
        || op.ends_with(".get")
        || op.ends_with(".history")
        || op.ends_with(".view")
        || op.ends_with(".verify")
        || op.ends_with(".preview")
        || op.ends_with(".status")
        || op.ends_with(".mine")
        || op.ends_with(".roster")
        || op.ends_with(".resume")
        || op.ends_with(".self")
        || op.ends_with(".access")
        || op.ends_with(".events")
        || op.starts_with("search.")
}

/// Send one request to a plugin over the RPC bus and return its reply payload.
/// Transport failures and plugin error replies both become `Err` (they surface
/// to the model identically as an `isError` tool result).
async fn dispatch_plugin(plugin: &str, op: &str, payload: Value) -> Result<Value, String> {
    match RequestClient::new(SOURCE)
        .with_timeout(RPC_TIMEOUT)
        .request(plugin, op, payload)
        .await
    {
        Err(err) => Err(format!("{op} failed: {err}")),
        Ok(reply) => match &reply.error {
            Some(err) => Err(format!(
                "the plugin rejected the request [{}]: {}",
                err.code, err.message
            )),
            None => Ok(reply.payload),
        },
    }
}

/// The `tools/list` payload: hand-written notification tools plus one generic
/// tool per catalogued plugin operation.
fn tool_list() -> Value {
    let mut tools: Vec<Value> = TOOLS
        .iter()
        .map(|spec| {
            json!({
                "name": spec.name,
                "description": spec.description,
                "inputSchema": (spec.schema)(),
                "annotations": { "readOnlyHint": is_read_only(spec.op) }
            })
        })
        .collect();

    tools.extend(GENERIC_TOOLS.iter().map(|(_, short, op)| {
        json!({
            "name": tool_name(short, op),
            "description": tool_description(short, op),
            "inputSchema": schema_pass_through(),
            "annotations": {
                "readOnlyHint": is_read_only(op),
                "destructiveHint": is_destructive(op),
            }
        })
    }));

    Value::Array(tools)
}

/// Execute a `tools/call`. `Err` is a protocol-level problem (unknown tool);
/// everything else is reported to the model as an `isError` tool result.
async fn call_tool(params: &Value, caller: &Caller) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| "missing required parameter 'name'".to_string())?;

    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    // Hand-written notification tools first, then the generic catalog.
    let result = if let Some(spec) = TOOLS.iter().find(|tool| tool.name == name) {
        let payload = match build_payload(spec, &args, caller) {
            Ok(payload) => payload,
            Err(err) => return Ok(tool_error(&err.to_string())),
        };
        dispatch_plugin(NOTIFICATIONS_PLUGIN, spec.op, payload).await
    } else if let Some((plugin, _short, op)) = GENERIC_TOOLS
        .iter()
        .find(|(_, short, op)| &tool_name(short, op) == name)
    {
        let payload = match build_generic_payload(op, &args, caller) {
            Ok(payload) => payload,
            Err(err) => return Ok(tool_error(&err.to_string())),
        };
        dispatch_plugin(plugin, op, payload).await
    } else {
        return Err(format!("unknown tool: {name}"));
    };

    match result {
        Err(err) => Ok(tool_error(&err)),
        Ok(payload) => Ok(tool_result(payload)),
    }
}

/// Handle one JSON-RPC message. `None` means "notification → no response body".
async fn handle_message(message: &Value, caller: &Caller) -> Option<Value> {
    // A message without `id` is a JSON-RPC notification (e.g.
    // `notifications/initialized`): it is executed but never answered.
    let id = message.get("id").cloned()?;

    let method = message
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or_default();
    let params = message.get("params").cloned().unwrap_or(Value::Null);

    if method.is_empty() {
        return Some(rpc_error(id, -32600, "invalid request: missing 'method'"));
    }

    match method {
        "initialize" => {
            let requested = params.get("protocolVersion").and_then(|v| v.as_str());
            let protocol = match requested {
                Some(version) if SUPPORTED_PROTOCOLS.contains(&version) => version,
                _ => LATEST_PROTOCOL,
            };
            Some(rpc_result(
                id,
                json!({
                    "protocolVersion": protocol,
                    "capabilities": { "tools": { "listChanged": false } },
                    "serverInfo": {
                        "name": "elupane",
                        "title": "E-Lupane platform",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "instructions": "Tools for every E-Lupane plugin operation: notifications, academics, analytics, assignments, auth, certificates, content, course management, enrollments, grades, institution management, learning progress, live classes, media, messaging, quizzes, search and support. The signed-in session token is injected automatically — never pass 'token' or 'admin_token'. Role and ownership checks are enforced by each plugin; destructive operations are marked and should be confirmed with the user first."
                }),
            ))
        }
        "ping" => Some(rpc_result(id, json!({}))),
        "tools/list" => Some(rpc_result(id, json!({ "tools": tool_list() }))),
        "tools/call" => match call_tool(&params, caller).await {
            Ok(result) => Some(rpc_result(id, result)),
            Err(err) => Some(rpc_error(id, -32602, &err)),
        },
        other => Some(rpc_error(id, -32601, &format!("method not found: {other}"))),
    }
}

// ---------------------------------------------------------------------------
// Axum handlers (Streamable HTTP transport)
// ---------------------------------------------------------------------------

/// `POST /api/mcp` — every JSON-RPC exchange (a single message or a batch).
pub async fn mcp_post(
    Query(params): Query<AuthParams>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let caller = match authenticate(&headers, &params).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };

    let initializing = body.get("method").and_then(|m| m.as_str()) == Some("initialize");

    let response_body = match body {
        // JSON-RPC batch: answer each message, dropping the notifications.
        Value::Array(messages) => {
            let mut responses = Vec::new();
            for message in messages {
                if let Some(response) = handle_message(&message, &caller).await {
                    responses.push(response);
                }
            }
            if responses.is_empty() {
                None
            } else {
                Some(Value::Array(responses))
            }
        }
        message => handle_message(&message, &caller).await,
    };

    let mut response = match response_body {
        Some(body) => json_response(StatusCode::OK, body),
        // Nothing but notifications was sent.
        None => accepted(),
    };

    // Clients that expect the optional session header get one on `initialize`;
    // nothing is stored server-side.
    if initializing {
        if let Ok(session_id) = HeaderValue::from_str(&Uuid::new_v4().to_string()) {
            response.headers_mut().insert("mcp-session-id", session_id);
        }
    }

    response
}

/// `GET /api/mcp` — no server-initiated SSE stream is offered, so the spec's
/// 405 answer applies.
pub async fn mcp_get() -> Response {
    let mut response = (
        StatusCode::METHOD_NOT_ALLOWED,
        "SSE streaming is not offered by this server; POST JSON-RPC to this endpoint.",
    )
        .into_response();
    response
        .headers_mut()
        .insert(header::ALLOW, HeaderValue::from_static("POST, DELETE, OPTIONS"));
    cors(response)
}

/// `DELETE /api/mcp` — session termination. The transport keeps no session
/// state, so this is always a no-op success.
pub async fn mcp_delete() -> Response {
    cors(StatusCode::NO_CONTENT.into_response())
}

/// `OPTIONS /api/mcp` — CORS preflight for browser-based MCP clients.
pub async fn mcp_options() -> Response {
    cors(StatusCode::NO_CONTENT.into_response())
}

/// 202 Accepted with an empty body (JSON-RPC notifications get no response).
fn accepted() -> Response {
    cors(StatusCode::ACCEPTED.into_response())
}

/// JSON body with the given status.
fn json_response(status: StatusCode, body: Value) -> Response {
    let mut response = Json(body).into_response();
    *response.status_mut() = status;
    cors(response)
}

/// JSON error body for failures that happen outside the JSON-RPC layer
/// (missing/invalid credentials, auth plugin unreachable).
fn error_response(status: StatusCode, message: &str) -> Response {
    json_response(status, json!({ "error": message }))
}

/// Permissive CORS: MCP clients authenticate with a bearer token (never
/// cookies), so a wildcard origin is safe here.
fn cors(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("POST, GET, DELETE, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static(
            "content-type, authorization, mcp-protocol-version, mcp-session-id",
        ),
    );
    headers.insert(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        HeaderValue::from_static("mcp-session-id"),
    );
    response
}
