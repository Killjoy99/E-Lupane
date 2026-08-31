use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::*;
use crate::services;
use crate::models::*;

pub const PLUGIN_ID: &str = "com.elupane.analytics";

pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "analytics.error", &err.to_string()),
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
        "analytics.student.progress" => {
            let _claims = authorize(&command.payload)?;
            let req: StudentCourseRequest = parse_payload(&command.payload)?;
            let view = services::get_student_progress(pool, req.student_id, req.course_id).await?;
            reply(command, &view)
        }
        "analytics.instructor.overview" => {
            let _claims = authorize(&command.payload)?;
            let req: StudentCourseRequest = parse_payload(&command.payload)?;
            let view = services::get_instructor_analytics(pool, req.student_id, req.course_id).await?;
            reply(command, &view)
        }
        "analytics.admin.overview" => {
            let claims = authorize(&command.payload)?;
            let req: DateRangeRequest = parse_payload(&command.payload)?;
            let institution_id = claims.user_id()?;
            let view = services::get_admin_analytics(pool, institution_id, req.start, req.end).await?;
            reply(command, &view)
        }
        "analytics.reports.generate" => {
            let claims = authorize(&command.payload)?;
            let req: GenerateReportRequest = parse_payload(&command.payload)?;
            let view = services::generate_report(pool, &claims, req).await?;
            reply(command, &view)
        }
        "analytics.reports.list" => {
            let claims = authorize(&command.payload)?;
            let view = services::list_reports(pool, Some(claims.user_id()?)).await?;
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
