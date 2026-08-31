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
