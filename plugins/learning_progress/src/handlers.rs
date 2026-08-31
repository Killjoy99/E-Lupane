use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::*;
use crate::services;
use crate::models::*;

pub const PLUGIN_ID: &str = "com.elupane.learning_progress";

pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "learning_progress.error", &err.to_string()),
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
        "learning_progress.get_or_create" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::get_or_create_progress(pool, &claims, req.course_id).await?;
            reply(command, &view)
        }
        "learning_progress.update" => {
            let claims = authorize(&command.payload)?;
            let req: StudentCourseRequest = parse_payload(&command.payload)?;
            let body: UpdateProgressRequest = parse_payload(&command.payload)?;
            let view = services::update_progress(pool, &claims, req.course_id, body).await?;
            reply(command, &view)
        }
        "learning_progress.mark_complete" => {
            let claims = authorize(&command.payload)?;
            let req: StudentCourseRequest = parse_payload(&command.payload)?;
            let body: MarkCompleteRequest = parse_payload(&command.payload)?;
            let view = services::mark_lesson_complete(pool, &claims, req.course_id, body).await?;
            reply(command, &view)
        }
        "learning_progress.course_summary" => {
            let claims = authorize(&command.payload)?;
            let req: StudentCourseRequest = parse_payload(&command.payload)?;
            let view = services::get_course_progress(pool, req.student_id, req.course_id).await?;
            reply(command, &view)
        }
        "learning_progress.resume" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::resume_learning(pool, &claims, req.course_id).await?;
            reply(command, &view)
        }
        "learning_progress.events" => {
            let claims = authorize(&command.payload)?;
            let req: LessonIdRequest = parse_payload(&command.payload)?;
            let view = services::get_progress_events(pool, req.lesson_id).await?;
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
