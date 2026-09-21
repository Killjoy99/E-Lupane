//! Types mirrored from the support plugin's views. Compiled on BOTH the client
//! (WASM) and the server so server functions and the dashboard can share them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupportTicketView {
    pub id: Uuid,
    pub subject: String,
    pub category: String,
    pub priority: String,
    pub status: String,
    pub requester_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupportMessageView {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupportTicketDetail {
    pub ticket: SupportTicketView,
    pub messages: Vec<SupportMessageView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupportFaqView {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub category: String,
    pub published: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateTicketInput {
    pub subject: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub priority: String,
    #[serde(default)]
    pub course_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFaqInput {
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub category: String,
}

pub const SUPPORT_PRIORITIES: [&str; 4] = ["low", "normal", "high", "urgent"];
pub const SUPPORT_STATUSES: [&str; 4] = ["open", "in_progress", "resolved", "closed"];
pub const SUPPORT_CATEGORIES: [&str; 5] = ["general", "login", "courses", "assignments", "technical"];
