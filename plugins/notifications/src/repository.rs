use sqlx::PgPool;
use uuid::Uuid;
use crate::models::*;

pub async fn create_notification(pool: &PgPool, notification: &Notification) -> anyhow::Result<Notification> {
    let row = sqlx::query_as::<_, Notification>(
        r#"INSERT INTO notifications (id, user_id, type, title, body, data, read, read_at, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id, user_id, type, title, body, data, read, read_at, created_at"#,
    )
    .bind(notification.id)
    .bind(notification.user_id)
    .bind(&notification.notification_type)
    .bind(&notification.title)
    .bind(&notification.body)
    .bind(&notification.data)
    .bind(notification.read)
    .bind(notification.read_at)
    .bind(notification.created_at)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_notification(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<Notification>> {
    let row = sqlx::query_as::<_, Notification>("SELECT * FROM notifications WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_user_notifications(pool: &PgPool, user_id: Uuid, unread_only: bool) -> anyhow::Result<Vec<Notification>> {
    let sql = if unread_only {
        "SELECT * FROM notifications WHERE user_id = $1 AND read = false ORDER BY created_at DESC"
    } else {
        "SELECT * FROM notifications WHERE user_id = $1 ORDER BY created_at DESC"
    };
    let rows = sqlx::query_as::<_, Notification>(sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn mark_read(pool: &PgPool, id: Uuid) -> anyhow::Result<Notification> {
    let row = sqlx::query_as::<_, Notification>(
        r#"UPDATE notifications SET read = true, read_at = now() WHERE id = $1
           RETURNING id, user_id, type, title, body, data, read, read_at, created_at"#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn mark_all_read(pool: &PgPool, user_id: Uuid) -> anyhow::Result<u64> {
    let result = sqlx::query("UPDATE notifications SET read = true, read_at = now() WHERE user_id = $1 AND read = false")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

pub async fn set_preference(pool: &PgPool, preference: &NotificationPreference) -> anyhow::Result<NotificationPreference> {
    let row = sqlx::query_as::<_, NotificationPreference>(
        r#"INSERT INTO notification_preferences (id, user_id, notification_type, channel, enabled, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (user_id, notification_type, channel) DO UPDATE SET enabled = EXCLUDED.enabled, updated_at = EXCLUDED.updated_at
           RETURNING id, user_id, notification_type, channel, enabled, updated_at"#,
    )
    .bind(preference.id)
    .bind(preference.user_id)
    .bind(&preference.notification_type)
    .bind(&preference.channel)
    .bind(preference.enabled)
    .bind(preference.updated_at)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_preferences(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<NotificationPreference>> {
    let rows = sqlx::query_as::<_, NotificationPreference>("SELECT * FROM notification_preferences WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn create_delivery(pool: &PgPool, delivery: &NotificationDelivery) -> anyhow::Result<NotificationDelivery> {
    let row = sqlx::query_as::<_, NotificationDelivery>(
        r#"INSERT INTO notification_deliveries (id, notification_id, channel, status, error_message, delivered_at, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, notification_id, channel, status, error_message, delivered_at, created_at"#,
    )
    .bind(delivery.id)
    .bind(delivery.notification_id)
    .bind(&delivery.channel)
    .bind(&delivery.status)
    .bind(&delivery.error_message)
    .bind(delivery.delivered_at)
    .bind(delivery.created_at)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update_delivery(pool: &PgPool, id: Uuid, status: &str, error_message: Option<&str>) -> anyhow::Result<NotificationDelivery> {
    let row = sqlx::query_as::<_, NotificationDelivery>(
        r#"UPDATE notification_deliveries SET status = $2, error_message = COALESCE($3, error_message), delivered_at = CASE WHEN $2 = 'sent' THEN now() ELSE delivered_at END WHERE id = $1
           RETURNING id, notification_id, channel, status, error_message, delivered_at, created_at"#,
    )
    .bind(id)
    .bind(status)
    .bind(error_message)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_deliveries(pool: &PgPool, notification_id: Uuid) -> anyhow::Result<Vec<NotificationDelivery>> {
    let rows = sqlx::query_as::<_, NotificationDelivery>("SELECT * FROM notification_deliveries WHERE notification_id = $1 ORDER BY created_at")
        .bind(notification_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Support-event recipients (shared database lookups)
// ---------------------------------------------------------------------------

/// Ids of all users holding at least one of `roles`.
pub async fn role_user_ids(pool: &PgPool, roles: &[&str]) -> anyhow::Result<Vec<Uuid>> {
    let roles: Vec<String> = roles.iter().map(|s| s.to_string()).collect();
    Ok(sqlx::query_scalar(
        "SELECT DISTINCT u.id \
         FROM users u \
         JOIN user_roles ur ON ur.user_id = u.id \
         JOIN roles r ON r.id = ur.role_id \
         WHERE r.name = ANY($1::text[])",
    )
    .bind(roles)
    .fetch_all(pool)
    .await?)
}

/// Whether `user_id` holds at least one of `roles`.
pub async fn user_has_role(pool: &PgPool, user_id: Uuid, roles: &[&str]) -> anyhow::Result<bool> {
    let roles: Vec<String> = roles.iter().map(|s| s.to_string()).collect();
    Ok(sqlx::query_scalar(
        "SELECT EXISTS( \
            SELECT 1 FROM user_roles ur \
            JOIN roles r ON r.id = ur.role_id \
            WHERE ur.user_id = $1 AND r.name = ANY($2::text[]) \
         )",
    )
    .bind(user_id)
    .bind(roles)
    .fetch_one(pool)
    .await?)
}

/// The requester of a support ticket (only if the support plugin's tables exist).
pub async fn support_ticket_requester(pool: &PgPool, ticket_id: Uuid) -> anyhow::Result<Option<Uuid>> {
    let exists: Option<String> = sqlx::query_scalar("SELECT to_regclass('public.support_tickets')::text")
        .fetch_one(pool)
        .await?;
    if exists.is_none() {
        return Ok(None);
    }
    Ok(sqlx::query_scalar("SELECT requester_id FROM support_tickets WHERE id = $1")
        .bind(ticket_id)
        .fetch_optional(pool)
        .await?)
}
