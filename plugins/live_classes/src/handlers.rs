//! Command dispatch: maps incoming envelope ops to services and builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::*;
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.live_classes";

pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "live_classes.error", &err.to_string()),
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
        "live.schedule" => {
            let claims = authorize(&command.payload)?;
            let req: ScheduleRequest = parse_payload(&command.payload)?;
            let view = services::schedule(pool, claims.user_id()?, claims.is_admin(), req).await?;
            reply(command, &view)
        }
        "live.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateRequest = parse_payload(&command.payload)?;
            let view = services::update(pool, claims.user_id()?, claims.is_admin(), req).await?;
            reply(command, &view)
        }
        "live.list" => {
            let _claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let sessions = services::list_for_course(pool, req.course_id).await?;
            reply(command, &SessionsView { sessions })
        }
        "live.get" => {
            let _claims = authorize(&command.payload)?;
            let req: SessionIdRequest = parse_payload(&command.payload)?;
            let view = services::get(pool, req.session_id).await?;
            reply(command, &view)
        }
        "live.join" => {
            let claims = authorize(&command.payload)?;
            let req: SessionIdRequest = parse_payload(&command.payload)?;
            let view = services::join(pool, claims.user_id()?, claims.is_admin(), req.session_id).await?;
            reply(command, &view)
        }
        "live.start" => {
            let claims = authorize(&command.payload)?;
            let req: SessionIdRequest = parse_payload(&command.payload)?;
            let view = services::start(pool, claims.user_id()?, claims.is_admin(), req.session_id).await?;
            reply(command, &view)
        }
        "live.end" => {
            let claims = authorize(&command.payload)?;
            let req: SessionIdRequest = parse_payload(&command.payload)?;
            let view = services::end(pool, claims.user_id()?, claims.is_admin(), req.session_id).await?;
            reply(command, &view)
        }
        "live.cancel" => {
            let claims = authorize(&command.payload)?;
            let req: SessionIdRequest = parse_payload(&command.payload)?;
            let view = services::cancel(pool, claims.user_id()?, claims.is_admin(), req.session_id).await?;
            reply(command, &view)
        }
        "live.leave" => {
            let _claims = authorize(&command.payload)?;
            let req: AttendanceIdRequest = parse_payload(&command.payload)?;
            services::leave(pool, req.attendance_id).await?;
            reply(command, &serde_json::json!({ "ok": true }))
        }
        "live.attendance.list" => {
            let claims = authorize(&command.payload)?;
            let req: SessionIdRequest = parse_payload(&command.payload)?;
            let attendance =
                services::list_attendance(pool, claims.user_id()?, claims.is_admin(), req.session_id).await?;
            reply(command, &AttendanceListView { attendance })
        }
        "live.chat.send" => {
            let claims = authorize(&command.payload)?;
            let req: ChatSendRequest = parse_payload(&command.payload)?;
            let view = services::send_chat(pool, claims.user_id()?, claims.is_admin(), req).await?;
            reply(command, &view)
        }
        "live.chat.history" => {
            let claims = authorize(&command.payload)?;
            let req: ChatHistoryRequest = parse_payload(&command.payload)?;
            let messages =
                services::chat_history(pool, claims.user_id()?, claims.is_admin(), req).await?;
            reply(command, &ChatHistoryView { messages })
        }
        "live.resource.attach" => {
            let claims = authorize(&command.payload)?;
            let req: AttachResourceRequest = parse_payload(&command.payload)?;
            let view = services::attach_resource(pool, claims.user_id()?, claims.is_admin(), req).await?;
            reply(command, &view)
        }
        "live.resource.list" => {
            let claims = authorize(&command.payload)?;
            let req: SessionIdRequest = parse_payload(&command.payload)?;
            let resources =
                services::list_resources(pool, claims.user_id()?, claims.is_admin(), req.session_id).await?;
            reply(command, &ResourcesView { resources })
        }
        "live.recording.save" => {
            let claims = authorize(&command.payload)?;
            let req: SaveRecordingRequest = parse_payload(&command.payload)?;
            let view = services::save_recording(pool, claims.user_id()?, claims.is_admin(), req).await?;
            reply(command, &view)
        }
        other => Err(anyhow!("unknown op '{other}'")),
    }
}
