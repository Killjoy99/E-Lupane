//! Types mirrored from the live_classes plugin's views. Compiled on BOTH the
//! client (WASM) and the server so server functions and the dashboard can
//! share them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveSessionView {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveJoinView {
    pub session: LiveSessionView,
    pub allowed: bool,
    pub role: String,
    pub attendance_id: Uuid,
    pub signaling_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveAttendanceView {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveChatMessageView {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sender_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveResourceView {
    pub id: Uuid,
    pub session_id: Uuid,
    pub name: String,
    pub url: String,
    pub kind: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Human-readable label for a session status.
pub fn status_label(status: &str) -> &'static str {
    match status {
        "scheduled" => "Scheduled",
        "live" => "Live",
        "ended" => "Ended",
        "cancelled" => "Cancelled",
        _ => "Unknown",
    }
}

/// Status-chip tone (matches the dashboard's StatusChip tones).
pub fn status_tone(status: &str) -> &'static str {
    match status {
        "live" => "ok",
        "scheduled" => "info",
        "ended" => "muted",
        "cancelled" => "muted",
        _ => "muted",
    }
}
