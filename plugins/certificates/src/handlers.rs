//! Command dispatch: maps incoming envelope ops to services and builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::{
    GenerateCertificateRequest, GetCertificateRequest, ListCertificatesRequest, VerifyCertificateRequest,
};
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.certificates";

/// Handle a command and produce a reply message (success or error).
pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "certificates.error", &err.to_string()),
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
        // ---- Certificate generation ----
        "certificate.generate" => {
            let claims = authorize(&command.payload)?;
            let req: GenerateCertificateRequest = parse_payload(&command.payload)?;
            let user_id = claims.user_id()?;
            let view = services::generate_certificate(pool, user_id, &req).await?;
            reply(command, &view)
        }

        // ---- Certificate queries ----
        "certificate.list" => {
            let claims = authorize(&command.payload)?;
            let req: ListCertificatesRequest = parse_payload(&command.payload)?;
            let user_id = claims.user_id()?;
            let view = services::list_certificates(pool, user_id, claims.is_admin(), &req).await?;
            reply(command, &view)
        }
        "certificate.get" => {
            let claims = authorize(&command.payload)?;
            let req: GetCertificateRequest = parse_payload(&command.payload)?;
            let user_id = claims.user_id()?;
            let view = services::get_certificate(pool, user_id, claims.is_admin(), &req.certificate_id).await?;
            reply(command, &view)
        }

        // ---- Public verification ----
        "certificate.verify" => {
            let req: VerifyCertificateRequest = parse_payload(&command.payload)?;
            let view = services::verify_certificate(pool, &req.certificate_code).await?;
            reply(command, &view)
        }

        _ => Err(anyhow!("unknown op '{}'", command.op)),
    }
}
