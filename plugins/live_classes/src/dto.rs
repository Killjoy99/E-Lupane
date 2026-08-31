//! Request/response payloads exchanged over the message envelope.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleRequest {
    pub course_id: Uuid,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_end: DateTime<Utc>,
    /// webrtc (default, in-app) | jitsi | zoom | teams
    #[serde(default = "default_provider")]
    pub provider: String,
    /// Required for external providers (jitsi/zoom/teams); ignored for webrtc.
    #[serde(default)]
    pub external_url: Option<String>,
}

fn default_provider() -> String {
    "webrtc".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRequest {
    pub session_id: Uuid,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub scheduled_start: Option<DateTime<Utc>>,
    #[serde(default)]
    pub scheduled_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionIdRequest {
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CourseIdRequest {
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttendanceIdRequest {
    pub attendance_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatSendRequest {
    pub session_id: Uuid,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatHistoryRequest {
    pub session_id: Uuid,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttachResourceRequest {
    pub session_id: Uuid,
    pub name: String,
    pub url: String,
    #[serde(default = "default_kind")]
    pub kind: String,
}

fn default_kind() -> String {
    "link".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveRecordingRequest {
    pub session_id: Uuid,
    pub url: String,
    #[serde(default)]
    pub duration_seconds: Option<i32>,
    #[serde(default)]
    pub size_bytes: Option<i64>,
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SessionView {
    pub id: Uuid,
    pub course_id: Uuid,
    pub title: String,
    pub description: String,
    pub host_id: Uuid,
    pub provider: String,
    pub external_url: Option<String>,
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_end: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionsView {
    pub sessions: Vec<SessionView>,
}

/// Returned by `live.join`: whether the caller may join, plus the WebSocket
/// signaling channel to connect to for webrtc-provider sessions.
#[derive(Debug, Clone, Serialize)]
pub struct JoinView {
    pub session: SessionView,
    pub allowed: bool,
    pub role: String,
    /// Attendance row id — pass to `live.leave` when the caller disconnects.
    pub attendance_id: Uuid,
    /// Relative WS path the frontend should connect to for webrtc signaling
    /// (chat + WebRTC offer/answer/ICE + whiteboard). `None` for external
    /// providers (jitsi/zoom/teams) — the frontend redirects to `external_url`.
    pub signaling_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttendanceView {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttendanceListView {
    pub attendance: Vec<AttendanceView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessageView {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sender_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatHistoryView {
    pub messages: Vec<ChatMessageView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceView {
    pub id: Uuid,
    pub session_id: Uuid,
    pub name: String,
    pub url: String,
    pub kind: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourcesView {
    pub resources: Vec<ResourceView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecordingView {
    pub id: Uuid,
    pub session_id: Uuid,
    pub url: String,
    pub duration_seconds: Option<i32>,
    pub size_bytes: Option<i64>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}
