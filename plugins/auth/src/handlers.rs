//! Command dispatch: maps incoming envelope ops to services and builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::dto::{AssignRoleRequest, BulkCreateUserRequest, CreateRoleRequest, CreateUserRequest, DeleteUserRequest, GetUserRequest, GoogleLoginRequest, LoginRequest, RevokeRoleRequest, SetRolePermissionsRequest, VerifyRequest};
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.auth";

/// Handle a command and produce a reply message (success or error).
pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "auth.error", &err.to_string()),
    }
}

async fn dispatch(pool: &PgPool, command: &Message) -> Result<Message> {
    match command.op.as_str() {
        "user.create" => {
            let mut req: CreateUserRequest = parse_payload(&command.payload)?;
            let token = req.admin_token.as_ref()
                .ok_or_else(|| anyhow!("admin_token required in request payload"))?;
            let claims = services::verify_token(token)?;
            if !claims.is_admin() {
                return Err(anyhow!("admin privileges required to create users"));
            }
            req.admin_token = None;
            let view = services::create_user(pool, req).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(view)?))
        }
        "user.bulk_create" => {
            let req: BulkCreateUserRequest = parse_payload(&command.payload)?;
            let token = req.users.first()
                .and_then(|u| u.admin_token.as_ref())
                .ok_or_else(|| anyhow!("admin_token required in request payload"))?;
            let claims = services::verify_token(token)?;
            if !claims.is_admin() {
                return Err(anyhow!("admin privileges required to create users"));
            }
            let mut inner = req;
            for u in &mut inner.users {
                u.admin_token = None;
            }
            let response = services::bulk_create_users(pool, inner).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(response)?))
        }
        "user.get" => {
            let req: GetUserRequest = parse_payload(&command.payload)?;
            let view = services::get_user(pool, req.user_id).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(view)?))
        }
        "user.list" => {
            let views = services::list_users(pool).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(views)?))
        }
        "user.delete" => {
            let req: DeleteUserRequest = parse_payload(&command.payload)?;
            let token = req.admin_token.as_ref()
                .ok_or_else(|| anyhow!("admin_token required in request payload"))?;
            let claims = services::verify_token(token)?;
            if !claims.is_admin() {
                return Err(anyhow!("admin privileges required to delete users"));
            }
            services::delete_user(pool, req.user_id).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::json!({"deleted": true})))
        }
        "auth.login" => {
            let req: LoginRequest = parse_payload(&command.payload)?;
            let resp = services::authenticate(pool, &req.username, &req.password).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(resp)?))
        }
        "auth.google_login" => {
            let req: GoogleLoginRequest = parse_payload(&command.payload)?;
            let resp = services::google_login(pool, req).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(resp)?))
        }
        "auth.verify" => {
            let req: VerifyRequest = parse_payload(&command.payload)?;
            let resp = services::verify(pool, &req.token).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(resp)?))
        }
        "role.assign" => {
            let req: AssignRoleRequest = parse_payload(&command.payload)?;
            require_admin(&command, &req.admin_token)?;
            let view = services::assign_role(pool, req.user_id, &req.role).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(view)?))
        }
        "role.list" => {
            let roles = services::list_roles(pool).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(roles)?))
        }
        "permission.list" => {
            let permissions = services::list_permissions(pool).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(permissions)?))
        }
        "role.revoke" => {
            let req: RevokeRoleRequest = parse_payload(&command.payload)?;
            require_admin(&command, &req.admin_token)?;
            let view = services::revoke_role(pool, req.user_id, &req.role).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(view)?))
        }
        "role.permissions.set" => {
            let req: SetRolePermissionsRequest = parse_payload(&command.payload)?;
            require_admin(&command, &req.admin_token)?;
            let role = services::set_role_permissions(pool, &req.role, req.permissions.clone()).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(role)?))
        }
        "role.create" => {
            let req: CreateRoleRequest = parse_payload(&command.payload)?;
            require_admin(&command, &req.admin_token)?;
            let role = services::create_role(pool, &req.name, req.description.clone(), req.permissions.clone()).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(role)?))
        }
        _ => Err(anyhow!("unknown operation: {}", command.op)),
    }
}

/// Validate the admin JWT carried in the request payload (`admin_token`).
fn require_admin(_command: &Message, token: &Option<String>) -> Result<()> {
    let token = token.as_ref()
        .ok_or_else(|| anyhow!("admin_token required in request payload"))?;
    let claims = services::verify_token(token)?;
    if !claims.is_admin() {
        return Err(anyhow!("admin privileges required"));
    }
    Ok(())
}

fn parse_payload<T: DeserializeOwned>(payload: &Value) -> Result<T> {
    serde_json::from_value(payload.clone()).map_err(|e| anyhow!("invalid payload: {e}"))
}
