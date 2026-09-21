//! Command dispatch: maps incoming envelope ops to services and builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::{
    CompleteUploadRequest, DeleteFileRequest, DownloadRequest, FileInfoRequest,
    ListFilesRequest, UploadRequest,
};
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.media_service";

/// Handle a command and produce a reply message (success or error).
pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "media_service.error", &err.to_string()),
    }
}

/// Extract the caller's JWT from the payload and verify it.
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
        // ---- Upload flow ----
        "media.upload.request" => {
            let claims = authorize(&command.payload)?;
            let req: UploadRequest = parse_payload(&command.payload)?;
            let user_id = claims.user_id()?;
            let view = services::request_upload(pool, user_id, &req).await?;
            reply(command, &view)
        }
        "media.upload.complete" => {
            let claims = authorize(&command.payload)?;
            let req: CompleteUploadRequest = parse_payload(&command.payload)?;
            let user_id = claims.user_id()?;
            let view = services::complete_upload(pool, user_id, &req).await?;
            reply(command, &view)
        }

        // ---- Download ----
        "media.download.request" => {
            let claims = authorize(&command.payload)?;
            let req: DownloadRequest = parse_payload(&command.payload)?;
            let user_id = claims.user_id()?;
            let view = services::request_download(pool, user_id, &req).await?;
            reply(command, &view)
        }

        // ---- File management ----
        "media.delete" => {
            let claims = authorize(&command.payload)?;
            let req: DeleteFileRequest = parse_payload(&command.payload)?;
            let user_id = claims.user_id()?;
            services::delete_file(pool, user_id, claims.is_admin(), &req.file_id).await?;
            reply(command, &serde_json::json!({ "deleted": true }))
        }
        "media.list" => {
            let claims = authorize(&command.payload)?;
            let req: ListFilesRequest = parse_payload(&command.payload)?;
            let user_id = claims.user_id()?;
            let view = services::list_files(pool, user_id, claims.is_admin(), &req).await?;
            reply(command, &view)
        }
        "media.get" => {
            let claims = authorize(&command.payload)?;
            let req: FileInfoRequest = parse_payload(&command.payload)?;
            let user_id = claims.user_id()?;
            let view = services::get_file_info(pool, user_id, claims.is_admin(), &req.file_id).await?;
            reply(command, &view)
        }

        _ => Err(anyhow!("unknown op '{}'", command.op)),
    }
}
