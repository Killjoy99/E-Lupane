use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::*;
use crate::services;
use crate::models::*;

pub const PLUGIN_ID: &str = "com.elupane.notifications";

pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "notifications.error", &err.to_string()),
    }
}

fn authorize(payload: &Value) -> Result<Claims> {
    let token = payload
        .get("token")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow!("missing 'token'"))?;
    authz::verify_token(token)
}

fn parse_payload<T: DeserializeOwned>(payload: &Value) -> Result<T> {
    serde_json::from_value(payload.clone()).map_err(|e| anyhow!("invalid payload: {e}"))
}

fn reply<T: serde::Serialize>(command: &Message, value: &T) -> Result<Message> {
    Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(value)?))
}

async fn dispatch(pool: &PgPool, command: &Message) -> Result<Message> {
    match command.op.as_str() {
        "notifications.send" => {
            let claims = authorize(&command.payload)?;
            let req: CreateNotificationRequest = parse_payload(&command.payload)?;
            let view = services::create_notification(pool, &claims, req).await?;
            reply(command, &view)
        }
        "notifications.list" => {
            let claims = authorize(&command.payload)?;
            let req: UserIdRequest = parse_payload(&command.payload)?;
            let view = services::list_user_notifications(pool, req.user_id, false).await?;
            reply(command, &view)
        }
        "notifications.unread" => {
            let claims = authorize(&command.payload)?;
            let req: UserIdRequest = parse_payload(&command.payload)?;
            let view = services::list_user_notifications(pool, req.user_id, true).await?;
            reply(command, &view)
        }
        "notifications.mark_read" => {
            let claims = authorize(&command.payload)?;
            let req: MarkReadRequest = parse_payload(&command.payload)?;
            let view = services::mark_read(pool, req.notification_id).await?;
            reply(command, &view)
        }
        "notifications.mark_all_read" => {
            let claims = authorize(&command.payload)?;
            let req: MarkAllReadRequest = parse_payload(&command.payload)?;
            let count = services::mark_all_read(pool, req.user_id).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::json!({ "marked_read": count })))
        }
        "notifications.preferences.set" => {
            let claims = authorize(&command.payload)?;
            let req: SetPreferenceRequest = parse_payload(&command.payload)?;
            let view = services::set_preference(pool, &claims, req).await?;
            reply(command, &view)
        }
        "notifications.preferences.list" => {
            let claims = authorize(&command.payload)?;
            let req: UserIdRequest = parse_payload(&command.payload)?;
            let view = services::list_preferences(pool, req.user_id).await?;
            reply(command, &view)
        }
        "notifications.deliveries" => {
            let claims = authorize(&command.payload)?;
            let req: IdRequest = parse_payload(&command.payload)?;
            let view = services::get_deliveries(pool, req.id).await?;
            reply(command, &view)
        }

        "events.subscribe" => Ok(Message::reply_to(
            command,
            PLUGIN_ID,
            serde_json::json!({ "subscribed": true }),
        )),

        _ => Err(anyhow!("unknown operation: {}", command.op)),
    }
}
