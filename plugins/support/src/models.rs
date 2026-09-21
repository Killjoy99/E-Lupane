//! Database row models for the support plugin.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct SupportTicket {
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

#[derive(Debug, Clone, FromRow)]
pub struct SupportMessage {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SupportFaq {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub category: String,
    pub created_by: Uuid,
    pub published: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
