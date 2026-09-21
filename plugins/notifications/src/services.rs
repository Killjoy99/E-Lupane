use uuid::Uuid;
use sqlx::PgPool;
use crate::authz::Claims;
use crate::dto::*;
use crate::models::*;
use crate::repository;
use crate::authz;

pub async fn create_notification(pool: &PgPool, claims: &Claims, req: CreateNotificationRequest) -> anyhow::Result<Notification> {
    authz::require_instructor(claims)?;
    let notification = Notification {
        id: uuid::Uuid::new_v4(),
        user_id: req.user_id,
        notification_type: req.notification_type,
        title: req.title,
        body: req.body.unwrap_or_default(),
        data: req.data.unwrap_or_else(|| serde_json::json!({})),
        read: false,
        read_at: None,
        created_at: chrono::Utc::now(),
    };
    repository::create_notification(pool, &notification).await
}

pub async fn get_notification(pool: &PgPool, notification_id: Uuid) -> anyhow::Result<Option<Notification>> {
    repository::get_notification(pool, notification_id).await
}

pub async fn list_user_notifications(pool: &PgPool, user_id: Uuid, unread_only: bool) -> anyhow::Result<Vec<Notification>> {
    repository::list_user_notifications(pool, user_id, unread_only).await
}

pub async fn mark_read(pool: &PgPool, notification_id: Uuid) -> anyhow::Result<Notification> {
    repository::mark_read(pool, notification_id).await
}

pub async fn mark_all_read(pool: &PgPool, user_id: Uuid) -> anyhow::Result<u64> {
    repository::mark_all_read(pool, user_id).await
}

pub async fn set_preference(pool: &PgPool, claims: &Claims, req: SetPreferenceRequest) -> anyhow::Result<NotificationPreference> {
    let preference = NotificationPreference {
        id: uuid::Uuid::new_v4(),
        user_id: claims.user_id()?,
        notification_type: req.notification_type,
        channel: req.channel,
        enabled: req.enabled,
        updated_at: chrono::Utc::now(),
    };
    repository::set_preference(pool, &preference).await
}

pub async fn list_preferences(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<NotificationPreference>> {
    repository::list_preferences(pool, user_id).await
}

pub async fn get_deliveries(pool: &PgPool, notification_id: Uuid) -> anyhow::Result<Vec<NotificationDelivery>> {
    repository::list_deliveries(pool, notification_id).await
}

pub async fn create_delivery(pool: &PgPool, delivery: &NotificationDelivery) -> anyhow::Result<NotificationDelivery> {
    repository::create_delivery(pool, delivery).await
}

pub async fn update_delivery(pool: &PgPool, id: Uuid, status: &str, error_message: Option<&str>) -> anyhow::Result<NotificationDelivery> {
    repository::update_delivery(pool, id, status, error_message).await
}

// ---------------------------------------------------------------------------
// Ingest of cross-plugin support events (fires when the support plugin posts
// an event on the `support` event channel).
// ---------------------------------------------------------------------------

const SUPPORT_STAFF: &[&str] = &["support", "admin", "super_admin"];
const SUPPORT_ADMINS: &[&str] = &["admin", "super_admin"];

/// Persist a single in-app notification (+ a "sent" in_app delivery row).
async fn persist(
    pool: &PgPool,
    user_id: Uuid,
    title: &str,
    body: &str,
) -> anyhow::Result<()> {
    let notification = Notification {
        id: uuid::Uuid::new_v4(),
        user_id,
        notification_type: "support".to_string(),
        title: title.to_string(),
        body: body.to_string(),
        data: serde_json::json!({}),
        read: false,
        read_at: None,
        created_at: chrono::Utc::now(),
    };
    let notification = repository::create_notification(pool, &notification).await?;
    let delivery = NotificationDelivery {
        id: uuid::Uuid::new_v4(),
        notification_id: notification.id,
        channel: "in_app".to_string(),
        status: "sent".to_string(),
        error_message: None,
        delivered_at: Some(chrono::Utc::now()),
        created_at: chrono::Utc::now(),
    };
    repository::create_delivery(pool, &delivery).await?;
    Ok(())
}

fn payload_uuid(payload: &serde_json::Value, key: &str) -> Option<Uuid> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
}

/// Handle a support-domain event: translate it into in-app notifications for the
/// right recipients (support staff on new tickets; the requester when staff
/// replies, and staff when a user replies; admins on FAQ changes).
pub async fn handle_support_event(pool: &PgPool, event: &elupane_core::message::Message) -> anyhow::Result<()> {
    let payload = &event.payload;
    match event.op.as_str() {
        "support.ticket.created" => {
            let subject = payload
                .get("subject")
                .and_then(|v| v.as_str())
                .unwrap_or("A new support ticket was opened");
            for uid in repository::role_user_ids(pool, SUPPORT_STAFF).await? {
                persist(pool, uid, "New support ticket", subject).await?;
            }
        }
        "support.message.created" => {
            let author = payload_uuid(payload, "author_id");
            if let Some(author_id) = author {
                let is_staff = repository::user_has_role(pool, author_id, SUPPORT_STAFF).await?;
                if is_staff {
                    // A staff member replied -> notify the ticket requester.
                    if let Some(ticket_id) = payload_uuid(payload, "ticket_id") {
                        if let Some(requester) = repository::support_ticket_requester(pool, ticket_id).await? {
                            persist(pool, requester, "Support reply", "A support representative replied to your ticket.").await?;
                        }
                    }
                } else {
                    // A user replied -> notify support staff.
                    for uid in repository::role_user_ids(pool, SUPPORT_STAFF).await? {
                        persist(pool, uid, "Support message", "A user replied to a support ticket.").await?;
                    }
                }
            }
        }
        "support.faq.created" => {
            for uid in repository::role_user_ids(pool, SUPPORT_ADMINS).await? {
                persist(pool, uid, "New FAQ article", "A new support article was published.").await?;
            }
        }
        _ => {}
    }
    Ok(())
}
