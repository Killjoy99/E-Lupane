//! Command dispatch: maps incoming envelope ops to services and builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::*;
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.support";

/// Handle a command and produce a reply message (success or error).
pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "support.error", &err.to_string()),
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
        // ---- Tickets ----
        "support.ticket.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateTicketRequest = parse_payload(&command.payload)?;
            let view = services::create_ticket(pool, &claims, req).await?;
            reply(command, &view)
        }
        "support.ticket.list" => {
            let claims = authorize(&command.payload)?;
            let view = services::list_tickets(pool, &claims).await?;
            reply(command, &view)
        }
        "support.ticket.get" => {
            let claims = authorize(&command.payload)?;
            let req: TicketIdRequest = parse_payload(&command.payload)?;
            let view = services::get_ticket(pool, &claims, req.ticket_id).await?;
            reply(command, &view)
        }
        "support.ticket.assign" => {
            let claims = authorize(&command.payload)?;
            let req: AssignTicketRequest = parse_payload(&command.payload)?;
            let view = services::assign_ticket(pool, &claims, req).await?;
            reply(command, &view)
        }
        "support.ticket.set_status" => {
            let claims = authorize(&command.payload)?;
            let req: SetTicketStatusRequest = parse_payload(&command.payload)?;
            let view = services::set_ticket_status(pool, &claims, req).await?;
            reply(command, &view)
        }

        // ---- Ticket thread ----
        "support.message.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateMessageRequest = parse_payload(&command.payload)?;
            let view = services::create_message(pool, &claims, req).await?;
            reply(command, &view)
        }

        // ---- FAQ ----
        "support.faq.list" => {
            let _claims = authorize(&command.payload)?;
            let view = services::list_faq(pool).await?;
            reply(command, &view)
        }
        "support.faq.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateFaqRequest = parse_payload(&command.payload)?;
            let view = services::create_faq(pool, &claims, req).await?;
            reply(command, &view)
        }
        "support.faq.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateFaqRequest = parse_payload(&command.payload)?;
            let view = services::update_faq(pool, &claims, req).await?;
            reply(command, &view)
        }
        "support.faq.delete" => {
            let claims = authorize(&command.payload)?;
            let req: FaqIdRequest = parse_payload(&command.payload)?;
            services::delete_faq(pool, &claims, req.faq_id).await?;
            Ok(Message::reply_to(
                command,
                PLUGIN_ID,
                serde_json::json!({ "deleted": true, "faq_id": req.faq_id }),
            ))
        }

        _ => Err(anyhow!("unknown operation: {}", command.op)),
    }
}
