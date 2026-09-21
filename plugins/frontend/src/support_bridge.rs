//! Server-only bridge between the Dioxus fullstack server and the support
//! plugin (`com.elupane.support`). Performs request/response RPC over the Redis
//! messaging layer, mirroring the other bridges.

use std::time::Duration;

use anyhow::{anyhow, Result};
use serde_json::json;

use messaging_core::rpc::RequestClient;

use crate::support_types::{
    CreateFaqInput, CreateTicketInput, SupportFaqView, SupportMessageView, SupportTicketDetail,
    SupportTicketView,
};

pub const SUPPORT_PLUGIN: &str = "com.elupane.support";
const SOURCE: &str = "com.elupane.frontend";

fn client() -> RequestClient {
    RequestClient::new(SOURCE).with_timeout(Duration::from_secs(8))
}

async fn request_typed<T: serde::de::DeserializeOwned>(
    op: &str,
    payload: serde_json::Value,
    what: &str,
) -> Result<T> {
    let reply = client().request(SUPPORT_PLUGIN, op, payload).await?;
    if let Some(err) = &reply.error {
        return Err(anyhow!("{}: {}", what, err.message));
    }
    serde_json::from_value(reply.payload)
        .map_err(|e| anyhow!("{} returned an unexpected payload: {e}", what))
}

fn token_payload(token: &str, extra: serde_json::Map<String, serde_json::Value>) -> serde_json::Value {
    let mut map = extra;
    map.insert("token".to_string(), json!(token));
    serde_json::Value::Object(map)
}

pub async fn create_ticket(token: &str, input: CreateTicketInput) -> Result<SupportTicketView> {
    let mut map = serde_json::to_value(input)?.as_object().cloned().unwrap_or_default();
    request_typed("support.ticket.create", token_payload(token, map), "support plugin").await
}

pub async fn list_tickets(token: &str) -> Result<Vec<SupportTicketView>> {
    let value: serde_json::Value =
        request_typed("support.ticket.list", token_payload(token, Default::default()), "support plugin").await?;
    let list = value
        .get("tickets")
        .cloned()
        .ok_or_else(|| anyhow!("support plugin reply missing 'tickets'"))?;
    serde_json::from_value(list).map_err(|e| anyhow!("invalid support tickets payload: {e}"))
}

pub async fn get_ticket(token: &str, ticket_id: uuid::Uuid) -> Result<SupportTicketDetail> {
    let mut map = serde_json::Map::new();
    map.insert("ticket_id".to_string(), json!(ticket_id));
    request_typed("support.ticket.get", token_payload(token, map), "support plugin").await
}

pub async fn set_ticket_status(token: &str, ticket_id: uuid::Uuid, status: &str) -> Result<SupportTicketView> {
    let mut map = serde_json::Map::new();
    map.insert("ticket_id".to_string(), json!(ticket_id));
    map.insert("status".to_string(), json!(status));
    request_typed("support.ticket.set_status", token_payload(token, map), "support plugin").await
}

pub async fn create_message(token: &str, ticket_id: uuid::Uuid, body: &str) -> Result<SupportMessageView> {
    let mut map = serde_json::Map::new();
    map.insert("ticket_id".to_string(), json!(ticket_id));
    map.insert("body".to_string(), json!(body));
    request_typed("support.message.create", token_payload(token, map), "support plugin").await
}

pub async fn list_faq(token: &str) -> Result<Vec<SupportFaqView>> {
    let value: serde_json::Value =
        request_typed("support.faq.list", token_payload(token, Default::default()), "support plugin").await?;
    let list = value
        .get("items")
        .cloned()
        .ok_or_else(|| anyhow!("support plugin reply missing 'items'"))?;
    serde_json::from_value(list).map_err(|e| anyhow!("invalid support faq payload: {e}"))
}

pub async fn create_faq(token: &str, input: CreateFaqInput) -> Result<SupportFaqView> {
    let mut map = serde_json::to_value(input)?.as_object().cloned().unwrap_or_default();
    request_typed("support.faq.create", token_payload(token, map), "support plugin").await
}

pub async fn assign_ticket(token: &str, ticket_id: uuid::Uuid, assignee_id: Option<uuid::Uuid>) -> Result<SupportTicketView> {
    let mut map = serde_json::Map::new();
    map.insert("ticket_id".to_string(), json!(ticket_id));
    map.insert("assignee_id".to_string(), json!(assignee_id));
    request_typed("support.ticket.assign", token_payload(token, map), "support plugin").await
}

pub async fn update_faq(
    token: &str,
    faq_id: uuid::Uuid,
    title: Option<String>,
    body: Option<String>,
    category: Option<String>,
    published: Option<bool>,
) -> Result<SupportFaqView> {
    let mut map = serde_json::Map::new();
    map.insert("faq_id".to_string(), json!(faq_id));
    if let Some(v) = title { map.insert("title".to_string(), json!(v)); }
    if let Some(v) = body { map.insert("body".to_string(), json!(v)); }
    if let Some(v) = category { map.insert("category".to_string(), json!(v)); }
    if let Some(v) = published { map.insert("published".to_string(), json!(v)); }
    request_typed("support.faq.update", token_payload(token, map), "support plugin").await
}

pub async fn delete_faq(token: &str, faq_id: uuid::Uuid) -> Result<()> {
    let mut map = serde_json::Map::new();
    map.insert("faq_id".to_string(), json!(faq_id));
    request_typed::<serde_json::Value>("support.faq.delete", token_payload(token, map), "support plugin").await?;
    Ok(())
}
