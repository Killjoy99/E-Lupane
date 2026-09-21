//! Request/response payloads exchanged over the message envelope.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    pub subject: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub priority: String,
    #[serde(default)]
    pub course_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct TicketIdRequest {
    pub ticket_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AssignTicketRequest {
    pub ticket_id: Uuid,
    /// None clears the assignee.
    pub assignee_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct SetTicketStatusRequest {
    pub ticket_id: Uuid,
    /// One of open | in_progress | resolved | closed
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub ticket_id: Uuid,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFaqRequest {
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFaqRequest {
    pub faq_id: Uuid,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub published: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FaqIdRequest {
    pub faq_id: Uuid,
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct TicketView {
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

#[derive(Debug, Serialize)]
pub struct MessageView {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TicketDetail {
    pub ticket: TicketView,
    pub messages: Vec<MessageView>,
}

#[derive(Debug, Serialize)]
pub struct FaqView {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub category: String,
    pub published: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TicketList {
    pub tickets: Vec<TicketView>,
}

#[derive(Debug, Serialize)]
pub struct FaqList {
    pub items: Vec<FaqView>,
}
