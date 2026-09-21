//! Command dispatch: maps incoming envelope ops to services and builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::*;
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.messaging";

pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "messaging.error", &err.to_string()),
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
        "messaging.discussion.list" => {
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let topics = services::list_topics(pool, req.course_id).await?;
            reply(command, &topics)
        }
        "messaging.discussion.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateTopicRequest = parse_payload(&command.payload)?;
            let topic = services::create_topic(pool, &claims, req).await?;
            reply(command, &topic)
        }
        "messaging.discussion.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateTopicRequest = parse_payload(&command.payload)?;
            let topic = services::update_topic(pool, &claims, req).await?;
            reply(command, &topic)
        }
        "messaging.discussion.delete" => {
            let claims = authorize(&command.payload)?;
            let req: IdRequest = parse_payload(&command.payload)?;
            services::delete_topic(pool, &claims, req.id).await?;
            reply(command, &serde_json::json!({ "deleted": true }))
        }
        "messaging.discussion.reply" => {
            let claims = authorize(&command.payload)?;
            let req: CreateReplyRequest = parse_payload(&command.payload)?;
            let new_reply = services::create_reply(pool, &claims, req).await?;
            reply(command, &new_reply)
        }
        "messaging.discussion.list_replies" => {
            let req: IdRequest = parse_payload(&command.payload)?;
            let replies = services::list_replies(pool, req.id).await?;
            reply(command, &replies)
        }
        "messaging.discussion.mark_answer" => {
            let claims = authorize(&command.payload)?;
            let topic_id = command.payload
                .get("topic_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| anyhow!("missing 'topic_id'"))?;
            let req: MarkAnswerRequest = parse_payload(&command.payload)?;
            let topic = services::mark_answer(pool, &claims, topic_id, req.reply_id).await?;
            reply(command, &topic)
        }
        _ => Err(anyhow!("unknown op '{}'", command.op)),
    }
}
