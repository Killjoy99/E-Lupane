//! Server-only bridge between the Dioxus fullstack server and the auth plugin.
//!
//! The browser never talks to Redis directly. Instead, the server functions in
//! [`crate::server_functions`] call into this module, which performs
//! request/response RPC against the auth plugin over the Redis messaging layer
//! (`messaging_core`) — exactly like any other plugin consumer would:
//!
//! ```text
//! browser ──HTTP──▶ frontend server ──Redis stream──▶ auth plugin
//!                        ▲                                  │
//!                        └───────── reply channel ──────────┘
//! ```

use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

use messaging_core::rpc::RequestClient;

/// The auth plugin's registered id (matches `plugins/auth/plugin.toml`).
pub const AUTH_PLUGIN: &str = "com.elupane.auth";

/// Who we are on the messaging layer.
const SOURCE: &str = "com.elupane.frontend";

/// User projection returned by the auth plugin (mirrors `UserView` there).
#[derive(Debug, Clone, Deserialize)]
pub struct UserView {
    pub id: String,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub active: bool,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    token: String,
    user: UserView,
}

#[derive(Debug, Deserialize)]
struct VerifyResponse {
    valid: bool,
    user: Option<UserView>,
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BulkCreateUserResponse {
    created: Vec<UserView>,
    errors: Vec<BulkCreateError>,
}

#[derive(Debug, Deserialize)]
struct BulkCreateError {
    index: usize,
    username: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateUserInput {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub roles: Vec<String>,
    pub admin_token: Option<String>,
}

/// Connect the shared Redis client. Idempotent; best-effort at boot so the
/// app still serves if Redis is unreachable (login will surface the error).
pub async fn init() {
    let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    if let Err(err) = cache::connection::connect(&url).await {
        eprintln!("[frontend] Redis unavailable at {url}: {err}");
    }
}

fn client() -> RequestClient {
    RequestClient::new(SOURCE).with_timeout(Duration::from_secs(8))
}

/// `auth.login` → `(token, user)` or an error carrying the plugin's message.
pub async fn login(username: &str, password: &str) -> Result<(String, UserView)> {
    if !cache::connection::is_initialized() {
        return Err(anyhow!("auth service is unavailable (Redis not connected)"));
    }

    let reply = client()
        .request(
            AUTH_PLUGIN,
            "auth.login",
            json!({ "username": username, "password": password }),
        )
        .await?;

    if let Some(err) = &reply.error {
        return Err(anyhow!("{}", err.message));
    }

    let resp: LoginResponse = serde_json::from_value(reply.payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected login payload"))?;
    Ok((resp.token, resp.user))
}

/// `auth.google_login` → `(token, user)` for Google OAuth sign-in. Creates the
/// user in the auth plugin's `users` table if the email is new.
pub async fn google_login(email: &str, full_name: &str, roles: Vec<String>) -> Result<(String, UserView)> {
    if !cache::connection::is_initialized() {
        return Err(anyhow!("auth service is unavailable (Redis not connected)"));
    }

    let reply = client()
        .request(
            AUTH_PLUGIN,
            "auth.google_login",
            json!({ "email": email, "full_name": full_name, "roles": roles }),
        )
        .await?;

    if let Some(err) = &reply.error {
        return Err(anyhow!("{}", err.message));
    }

    let resp: LoginResponse = serde_json::from_value(reply.payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected google_login payload"))?;
    Ok((resp.token, resp.user))
}

/// `auth.verify` → `Some(user)` when the token is valid, `None` otherwise.
pub async fn verify(token: &str) -> Result<Option<UserView>> {
    if !cache::connection::is_initialized() {
        return Err(anyhow!("auth service is unavailable (Redis not connected)"));
    }

    let reply = client()
        .request(AUTH_PLUGIN, "auth.verify", json!({ "token": token }))
        .await?;

    if let Some(err) = &reply.error {
        return Err(anyhow!("{}", err.message));
    }

    let resp: VerifyResponse = serde_json::from_value(reply.payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected verify payload"))?;
    if resp.valid {
        Ok(resp.user)
    } else {
        Ok(None)
    }
}

/// Create a new user account (admin only).
/// 
/// Requires an admin token for authorization.
pub async fn create_user(
    admin_token: &str,
    username: &str,
    email: &str,
    password: &str,
    full_name: &str,
    roles: Vec<String>,
) -> Result<UserView> {
    if !cache::connection::is_initialized() {
        return Err(anyhow!("auth service is unavailable (Redis not connected)"));
    }

    let reply = client()
        .request(
            AUTH_PLUGIN,
            "user.create",
            json!({
                "username": username,
                "email": email,
                "password": password,
                "full_name": full_name,
                "roles": roles,
                "admin_token": admin_token
            }),
        )
        .await?;

    if let Some(err) = &reply.error {
        return Err(anyhow!("{}", err.message));
    }

    let user: UserView = serde_json::from_value(reply.payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected create_user payload"))?;
    Ok(user)
}


/// Delete an existing user account (admin only).
pub async fn delete_user(admin_token: &str, user_id: &str) -> Result<()> {
    if !cache::connection::is_initialized() {
        return Err(anyhow!("auth service is unavailable (Redis not connected)"));
    }

    let reply = client()
        .request(
            AUTH_PLUGIN,
            "user.delete",
            json!({
                "user_id": user_id,
                "admin_token": admin_token
            }),
        )
        .await?;

    if let Some(err) = &reply.error {
        return Err(anyhow!("{}", err.message));
    }

    Ok(())
}

/// List all users (admin only).
pub async fn list_users(admin_token: &str) -> Result<Vec<UserView>> {
    if !cache::connection::is_initialized() {
        return Err(anyhow!("auth service is unavailable (Redis not connected)"));
    }

    let reply = client()
        .request(
            AUTH_PLUGIN,
            "user.list",
            json!({ "admin_token": admin_token }),
        )
        .await?;

    if let Some(err) = &reply.error {
        return Err(anyhow!("{}", err.message));
    }

    let users: Vec<UserView> = serde_json::from_value(reply.payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected list_users payload"))?;
    Ok(users)
}

/// Bulk create user accounts (admin only).
pub async fn bulk_create_users(
    admin_token: &str,
    users: Vec<CreateUserInput>,
) -> Result<Vec<UserView>> {
    if !cache::connection::is_initialized() {
        return Err(anyhow!("auth service is unavailable (Redis not connected)"));
    }

    let mut user_reqs = users;
    for u in &mut user_reqs {
        u.admin_token = Some(admin_token.to_string());
    }

    let reply = client()
        .request(
            AUTH_PLUGIN,
            "user.bulk_create",
            json!({ "users": user_reqs }),
        )
        .await?;

    if let Some(err) = &reply.error {
        return Err(anyhow!("{}", err.message));
    }

    let resp: BulkCreateUserResponse = serde_json::from_value(reply.payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected bulk_create_users payload"))?;

    if !resp.errors.is_empty() {
        let msgs: Vec<String> = resp.errors.iter().map(|e| format!("{}: {}", e.username, e.message)).collect();
        return Err(anyhow!("bulk import completed with errors: {}", msgs.join("; ")));
    }

    Ok(resp.created)
}

/// Valid account types (roles) that can be assigned to users when creating accounts.
pub const ACCOUNT_TYPES: &[&str] = &["student", "lecturer", "admin", "registrar", "manager", "developer"];

// ---------------------------------------------------------------------------
// Role / permission administration (RBAC matrix + user role management)
// ---------------------------------------------------------------------------

/// A role with its granted permissions (from `role.list`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoleView {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

/// A permission definition (from `permission.list`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PermissionView {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

async fn rpc(op: &str, payload: serde_json::Value) -> Result<serde_json::Value> {
    if !cache::connection::is_initialized() {
        return Err(anyhow!("auth service is unavailable (Redis not connected)"));
    }
    let reply = client().request(AUTH_PLUGIN, op, payload).await?;
    if let Some(err) = &reply.error {
        return Err(anyhow!("{}", err.message));
    }
    Ok(reply.payload)
}

/// `role.list` — every role with its permission set.
pub async fn list_roles() -> Result<Vec<RoleView>> {
    let payload = rpc("role.list", json!({})).await?;
    serde_json::from_value(payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected role.list payload"))
}

/// `permission.list` — every permission in the catalog.
pub async fn list_permissions() -> Result<Vec<PermissionView>> {
    let payload = rpc("permission.list", json!({})).await?;
    serde_json::from_value(payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected permission.list payload"))
}

/// `role.assign` — grant `role` to a user (admin only).
pub async fn grant_role(admin_token: &str, user_id: &str, role: &str) -> Result<UserView> {
    let payload = rpc(
        "role.assign",
        json!({ "user_id": user_id, "role": role, "admin_token": admin_token }),
    )
    .await?;
    serde_json::from_value(payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected role.assign payload"))
}

/// `role.revoke` — take `role` away from a user (admin only).
pub async fn revoke_role(admin_token: &str, user_id: &str, role: &str) -> Result<UserView> {
    let payload = rpc(
        "role.revoke",
        json!({ "user_id": user_id, "role": role, "admin_token": admin_token }),
    )
    .await?;
    serde_json::from_value(payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected role.revoke payload"))
}

/// `role.permissions.set` — replace a role's whole permission set (admin only).
pub async fn set_role_permissions(
    admin_token: &str,
    role: &str,
    permissions: Vec<String>,
) -> Result<RoleView> {
    let payload = rpc(
        "role.permissions.set",
        json!({ "role": role, "permissions": permissions, "admin_token": admin_token }),
    )
    .await?;
    serde_json::from_value(payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected role.permissions.set payload"))
}

/// `role.create` — define a new role, optionally with permissions (admin only).
pub async fn create_role(
    admin_token: &str,
    name: &str,
    description: Option<String>,
    permissions: Vec<String>,
) -> Result<RoleView> {
    let payload = rpc(
        "role.create",
        json!({
            "name": name,
            "description": description,
            "permissions": permissions,
            "admin_token": admin_token
        }),
    )
    .await?;
    serde_json::from_value(payload)
        .map_err(|_| anyhow!("auth plugin returned an unexpected role.create payload"))
}
