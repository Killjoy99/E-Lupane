//! Command dispatch: maps incoming envelope ops to services and builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::{
    CampusIdRequest, CreateCampusRequest, CreateInstitutionRequest, InstitutionIdRequest,
    ListCampusesRequest, UpdateCampusRequest, UpdateInstitutionRequest,
};
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.institution_management";

/// Handle a command and produce a reply message (success or error).
pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "institution.error", &err.to_string()),
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
        // ---- Institutions ----
        "institution.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateInstitutionRequest = parse_payload(&command.payload)?;
            reply(command, &services::create_institution(pool, &claims, req).await?)
        }
        "institution.get" => {
            let _claims = authorize(&command.payload)?;
            let req: InstitutionIdRequest = parse_payload(&command.payload)?;
            reply(command, &services::get_institution(pool, req.institution_id).await?)
        }
        "institution.list" => {
            let _claims = authorize(&command.payload)?;
            reply(command, &services::list_institutions(pool).await?)
        }
        "institution.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateInstitutionRequest = parse_payload(&command.payload)?;
            reply(command, &services::update_institution(pool, &claims, req).await?)
        }
        "institution.delete" => {
            let claims = authorize(&command.payload)?;
            let req: InstitutionIdRequest = parse_payload(&command.payload)?;
            services::delete_institution(pool, &claims, req.institution_id).await?;
            Ok(Message::reply_to(
                command,
                PLUGIN_ID,
                serde_json::json!({ "deleted": true, "institution_id": req.institution_id }),
            ))
        }

        // ---- Campuses ----
        "campus.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateCampusRequest = parse_payload(&command.payload)?;
            reply(command, &services::create_campus(pool, &claims, req).await?)
        }
        "campus.get" => {
            let _claims = authorize(&command.payload)?;
            let req: CampusIdRequest = parse_payload(&command.payload)?;
            reply(command, &services::get_campus(pool, req.campus_id).await?)
        }
        "campus.list" => {
            let _claims = authorize(&command.payload)?;
            let req: ListCampusesRequest = parse_payload(&command.payload)?;
            reply(command, &services::list_campuses(pool, req.institution_id).await?)
        }
        "campus.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateCampusRequest = parse_payload(&command.payload)?;
            reply(command, &services::update_campus(pool, &claims, req).await?)
        }
        "campus.delete" => {
            let claims = authorize(&command.payload)?;
            let req: CampusIdRequest = parse_payload(&command.payload)?;
            services::delete_campus(pool, &claims, req.campus_id).await?;
            Ok(Message::reply_to(
                command,
                PLUGIN_ID,
                serde_json::json!({ "deleted": true, "campus_id": req.campus_id }),
            ))
        }

        _ => Err(anyhow!("unknown operation: {}", command.op)),
    }
}
