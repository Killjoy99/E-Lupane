//! Request/response payloads exchanged over the message envelope.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: String,
    #[serde(default)]
    pub roles: Vec<String>,
    /// Admin JWT token required for creating new users.
    #[serde(default)]
    pub admin_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetUserRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct UserView {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub active: bool,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleLoginRequest {
    pub email: String,
    pub full_name: String,
    #[serde(default)]
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserView,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub user: Option<UserView>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteUserRequest {
    pub user_id: Uuid,
    /// Admin JWT token (required for user deletion).
    #[serde(default)]
    pub admin_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignRoleRequest {
    pub user_id: Uuid,
    pub role: String,
    /// Admin JWT token (required for role grant/revoke).
    #[serde(default)]
    pub admin_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RevokeRoleRequest {
    pub user_id: Uuid,
    pub role: String,
    #[serde(default)]
    pub admin_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetRolePermissionsRequest {
    pub role: String,
    pub permissions: Vec<String>,
    #[serde(default)]
    pub admin_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub admin_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkCreateUserRequest {
    pub users: Vec<CreateUserRequest>,
}

#[derive(Debug, Serialize)]
pub struct BulkCreateUserResponse {
    pub created: Vec<UserView>,
    pub errors: Vec<BulkCreateError>,
}

#[derive(Debug, Serialize)]
pub struct BulkCreateError {
    pub index: usize,
    pub username: String,
    pub message: String,
}
