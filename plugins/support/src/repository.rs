//! SQLx data access for the support plugin (runtime queries, no macros).

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{SupportFaq, SupportMessage, SupportTicket};

// ---------------------------------------------------------------------------
// Tickets
// ---------------------------------------------------------------------------

pub async fn insert_ticket(
    pool: &PgPool,
    subject: &str,
    category: &str,
    priority: &str,
    requester_id: Uuid,
    course_id: Option<Uuid>,
) -> Result<SupportTicket> {
    Ok(sqlx::query_as::<_, SupportTicket>(
        "INSERT INTO support_tickets (subject, category, priority, requester_id, course_id) \
         VALUES ($1,$2,$3,$4,$5) RETURNING id, subject, category, priority, status, requester_id, \
                 assignee_id, course_id, created_at, updated_at, closed_at",
    )
    .bind(subject)
    .bind(category)
    .bind(priority)
    .bind(requester_id)
    .bind(course_id)
    .fetch_one(pool)
    .await?)
}

pub async fn list_tickets_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<SupportTicket>> {
    Ok(sqlx::query_as::<_, SupportTicket>(
        "SELECT id, subject, category, priority, status, requester_id, assignee_id, course_id, \
                created_at, updated_at, closed_at \
         FROM support_tickets WHERE requester_id = $1 ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn list_all_tickets(pool: &PgPool) -> Result<Vec<SupportTicket>> {
    Ok(sqlx::query_as::<_, SupportTicket>(
        "SELECT id, subject, category, priority, status, requester_id, assignee_id, course_id, \
                created_at, updated_at, closed_at \
         FROM support_tickets ORDER BY updated_at DESC",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn find_ticket(pool: &PgPool, ticket_id: Uuid) -> Result<Option<SupportTicket>> {
    Ok(sqlx::query_as::<_, SupportTicket>(
        "SELECT id, subject, category, priority, status, requester_id, assignee_id, course_id, \
                created_at, updated_at, closed_at \
         FROM support_tickets WHERE id = $1",
    )
    .bind(ticket_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn assign_ticket(pool: &PgPool, ticket_id: Uuid, assignee_id: Option<Uuid>) -> Result<SupportTicket> {
    Ok(sqlx::query_as::<_, SupportTicket>(
        "UPDATE support_tickets SET assignee_id = $2, updated_at = now() \
         WHERE id = $1 RETURNING id, subject, category, priority, status, requester_id, \
                 assignee_id, course_id, created_at, updated_at, closed_at",
    )
    .bind(ticket_id)
    .bind(assignee_id)
    .fetch_one(pool)
    .await?)
}

pub async fn set_ticket_status(pool: &PgPool, ticket_id: Uuid, status: &str) -> Result<SupportTicket> {
    Ok(sqlx::query_as::<_, SupportTicket>(
        "UPDATE support_tickets SET status = $2, \
            closed_at = CASE WHEN $2 = 'closed' THEN now() ELSE NULL END, \
            updated_at = now() \
         WHERE id = $1 RETURNING id, subject, category, priority, status, requester_id, \
                 assignee_id, course_id, created_at, updated_at, closed_at",
    )
    .bind(ticket_id)
    .bind(status)
    .fetch_one(pool)
    .await?)
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

pub async fn insert_message(
    pool: &PgPool,
    ticket_id: Uuid,
    author_id: Uuid,
    body: &str,
) -> Result<SupportMessage> {
    Ok(sqlx::query_as::<_, SupportMessage>(
        "INSERT INTO support_messages (ticket_id, author_id, body) \
         VALUES ($1,$2,$3) RETURNING id, ticket_id, author_id, body, created_at",
    )
    .bind(ticket_id)
    .bind(author_id)
    .bind(body)
    .fetch_one(pool)
    .await?)
}

pub async fn list_messages(pool: &PgPool, ticket_id: Uuid) -> Result<Vec<SupportMessage>> {
    Ok(sqlx::query_as::<_, SupportMessage>(
        "SELECT id, ticket_id, author_id, body, created_at \
         FROM support_messages WHERE ticket_id = $1 ORDER BY created_at ASC",
    )
    .bind(ticket_id)
    .fetch_all(pool)
    .await?)
}

// ---------------------------------------------------------------------------
// FAQ
// ---------------------------------------------------------------------------

pub async fn list_faq(pool: &PgPool, published_only: bool) -> Result<Vec<SupportFaq>> {
    let rows = if published_only {
        sqlx::query_as::<_, SupportFaq>(
            "SELECT id, title, body, category, created_by, published, sort_order, created_at, updated_at \
             FROM support_faq WHERE published = TRUE ORDER BY sort_order ASC, created_at ASC",
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, SupportFaq>(
            "SELECT id, title, body, category, created_by, published, sort_order, created_at, updated_at \
             FROM support_faq ORDER BY sort_order ASC, created_at ASC",
        )
        .fetch_all(pool)
        .await?
    };
    Ok(rows)
}

pub async fn find_faq(pool: &PgPool, faq_id: Uuid) -> Result<Option<SupportFaq>> {
    Ok(sqlx::query_as::<_, SupportFaq>(
        "SELECT id, title, body, category, created_by, published, sort_order, created_at, updated_at \
         FROM support_faq WHERE id = $1",
    )
    .bind(faq_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn insert_faq(
    pool: &PgPool,
    title: &str,
    body: &str,
    category: &str,
    created_by: Uuid,
) -> Result<SupportFaq> {
    Ok(sqlx::query_as::<_, SupportFaq>(
        "INSERT INTO support_faq (title, body, category, created_by) \
         VALUES ($1,$2,$3,$4) RETURNING id, title, body, category, created_by, published, \
                 sort_order, created_at, updated_at",
    )
    .bind(title)
    .bind(body)
    .bind(category)
    .bind(created_by)
    .fetch_one(pool)
    .await?)
}

pub async fn update_faq(
    pool: &PgPool,
    faq_id: Uuid,
    title: Option<&str>,
    body: Option<&str>,
    category: Option<&str>,
    published: Option<bool>,
    sort_order: Option<i32>,
) -> Result<SupportFaq> {
    Ok(sqlx::query_as::<_, SupportFaq>(
        "UPDATE support_faq SET \
            title = COALESCE($2, title), \
            body = COALESCE($3, body), \
            category = COALESCE($4, category), \
            published = COALESCE($5, published), \
            sort_order = COALESCE($6, sort_order), \
            updated_at = now() \
         WHERE id = $1 RETURNING id, title, body, category, created_by, published, \
                 sort_order, created_at, updated_at",
    )
    .bind(faq_id)
    .bind(title)
    .bind(body)
    .bind(category)
    .bind(published)
    .bind(sort_order)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_faq(pool: &PgPool, faq_id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM support_faq WHERE id = $1")
        .bind(faq_id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

