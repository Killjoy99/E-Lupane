//! Business logic for the support plugin.

use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authz::{self, Claims};
use crate::dto::*;
use crate::handlers::PLUGIN_ID;
use crate::models::{SupportFaq, SupportMessage, SupportTicket};
use crate::repository;

/// Best-effort fire-and-forget event on the `support` event channel so the
/// notifications plugin can alert the right people. Never fails the operation.
async fn emit_event(op: &str, payload: serde_json::Value) {
    let event = elupane_core::message::Message::event(op, PLUGIN_ID, payload);
    let _ = messaging_core::events::publish_event("support", &event).await;
}

fn normalize_priority(p: &str) -> Result<&'static str> {
    let p = if p.trim().is_empty() { "normal" } else { p.trim() };
    match p {
        "low" => Ok("low"),
        "normal" => Ok("normal"),
        "high" => Ok("high"),
        "urgent" => Ok("urgent"),
        _ => Err(anyhow!("invalid priority '{p}' (expected low|normal|high|urgent)")),
    }
}

fn normalize_status(s: &str) -> Result<&'static str> {
    match s {
        "open" => Ok("open"),
        "in_progress" => Ok("in_progress"),
        "resolved" => Ok("resolved"),
        "closed" => Ok("closed"),
        _ => Err(anyhow!("invalid status '{s}' (expected open|in_progress|resolved|closed)")),
    }
}

fn category_or(v: &str) -> String {
    if v.trim().is_empty() { "general".to_string() } else { v.trim().to_string() }
}

fn to_ticket_view(t: &SupportTicket) -> TicketView {
    TicketView {
        id: t.id,
        subject: t.subject.clone(),
        category: t.category.clone(),
        priority: t.priority.clone(),
        status: t.status.clone(),
        requester_id: t.requester_id,
        assignee_id: t.assignee_id,
        course_id: t.course_id,
        created_at: t.created_at,
        updated_at: t.updated_at,
        closed_at: t.closed_at,
    }
}

fn to_message_view(m: &SupportMessage) -> MessageView {
    MessageView {
        id: m.id,
        ticket_id: m.ticket_id,
        author_id: m.author_id,
        body: m.body.clone(),
        created_at: m.created_at,
    }
}

fn to_faq_view(f: &SupportFaq) -> FaqView {
    FaqView {
        id: f.id,
        title: f.title.clone(),
        body: f.body.clone(),
        category: f.category.clone(),
        published: f.published,
        sort_order: f.sort_order,
        created_at: f.created_at,
        updated_at: f.updated_at,
    }
}

// ---------------------------------------------------------------------------
// Tickets
// ---------------------------------------------------------------------------

pub async fn create_ticket(pool: &PgPool, claims: &Claims, req: CreateTicketRequest) -> Result<TicketView> {
    if req.subject.trim().is_empty() {
        return Err(anyhow!("ticket subject must not be empty"));
    }
    let priority = normalize_priority(&req.priority)?;
    let category = category_or(&req.category);
    let me = claims.user_id()?;
    let t = repository::insert_ticket(pool, req.subject.trim(), &category, priority, me, req.course_id).await?;
    emit_event(
        "support.ticket.created",
        serde_json::json!({ "ticket_id": t.id, "requester_id": me, "subject": t.subject }),
    )
    .await;
    Ok(to_ticket_view(&t))
}

pub async fn list_tickets(pool: &PgPool, claims: &Claims) -> Result<TicketList> {
    let me = claims.user_id()?;
    let tickets = if claims.is_staff() {
        repository::list_all_tickets(pool).await?
    } else {
        repository::list_tickets_for_user(pool, me).await?
    };
    Ok(TicketList { tickets: tickets.iter().map(to_ticket_view).collect() })
}

pub async fn get_ticket(pool: &PgPool, claims: &Claims, ticket_id: Uuid) -> Result<TicketDetail> {
    let me = claims.user_id()?;
    let t = repository::find_ticket(pool, ticket_id)
        .await?
        .ok_or_else(|| anyhow!("ticket not found"))?;
    if !claims.is_staff() && t.requester_id != me {
        return Err(anyhow!("forbidden: ticket belongs to another user"));
    }
    let messages = repository::list_messages(pool, ticket_id).await?;
    Ok(TicketDetail {
        ticket: to_ticket_view(&t),
        messages: messages.iter().map(to_message_view).collect(),
    })
}

pub async fn assign_ticket(pool: &PgPool, claims: &Claims, req: AssignTicketRequest) -> Result<TicketView> {
    authz::require_staff(claims)?;
    let t = repository::assign_ticket(pool, req.ticket_id, req.assignee_id).await?;
    Ok(to_ticket_view(&t))
}

pub async fn set_ticket_status(pool: &PgPool, claims: &Claims, req: SetTicketStatusRequest) -> Result<TicketView> {
    authz::require_staff(claims)?;
    let status = normalize_status(req.status.trim())?;
    let t = repository::set_ticket_status(pool, req.ticket_id, status).await?;
    Ok(to_ticket_view(&t))
}

pub async fn create_message(pool: &PgPool, claims: &Claims, req: CreateMessageRequest) -> Result<MessageView> {
    let me = claims.user_id()?;
    let t = repository::find_ticket(pool, req.ticket_id)
        .await?
        .ok_or_else(|| anyhow!("ticket not found"))?;
    if !claims.is_staff() && t.requester_id != me {
        return Err(anyhow!("forbidden: ticket belongs to another user"));
    }
    if req.body.trim().is_empty() {
        return Err(anyhow!("message body must not be empty"));
    }
    let m = repository::insert_message(pool, req.ticket_id, me, req.body.trim()).await?;
    emit_event(
        "support.message.created",
        serde_json::json!({ "ticket_id": req.ticket_id, "author_id": me }),
    )
    .await;
    Ok(to_message_view(&m))
}

// ---------------------------------------------------------------------------
// FAQ
// ---------------------------------------------------------------------------

pub async fn list_faq(pool: &PgPool) -> Result<FaqList> {
    let items = repository::list_faq(pool, true).await?;
    Ok(FaqList { items: items.iter().map(to_faq_view).collect() })
}

pub async fn create_faq(pool: &PgPool, claims: &Claims, req: CreateFaqRequest) -> Result<FaqView> {
    authz::require_staff(claims)?;
    if req.title.trim().is_empty() {
        return Err(anyhow!("FAQ title must not be empty"));
    }
    let me = claims.user_id()?;
    let f = repository::insert_faq(pool, req.title.trim(), req.body.trim(), &category_or(&req.category), me).await?;
    // Publish immediately (no separate publish step in the first-pass UI).
    let f = repository::update_faq(pool, f.id, None, None, None, Some(true), None).await?;
    emit_event("support.faq.created", serde_json::json!({ "faq_id": f.id })).await;
    Ok(to_faq_view(&f))
}

pub async fn update_faq(pool: &PgPool, claims: &Claims, req: UpdateFaqRequest) -> Result<FaqView> {
    authz::require_staff(claims)?;
    let f = repository::update_faq(
        pool,
        req.faq_id,
        req.title.as_deref(),
        req.body.as_deref(),
        req.category.as_deref(),
        req.published,
        req.sort_order,
    )
    .await?;
    Ok(to_faq_view(&f))
}

pub async fn delete_faq(pool: &PgPool, claims: &Claims, faq_id: Uuid) -> Result<()> {
    authz::require_staff(claims)?;
    repository::delete_faq(pool, faq_id).await?;
    Ok(())
}

