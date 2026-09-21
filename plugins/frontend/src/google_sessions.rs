//! Redis-backed session management for Google OAuth.
//!
//! NOTE: the current Google sign-in flow mints short-lived JWTs via the auth
//! plugin (`google_classroom_bridge` -> `auth_bridge::google_login`); it does
//! NOT use opaque Redis tokens. This module is kept only as a self-contained
//! server-side session store for callers that opt into it, and must be
//! declared behind `#[cfg(feature = "server")]` (it needs Redis + Postgres).

use anyhow::Result;
use uuid::Uuid;

use crate::google_classroom_bridge::LocalUser;

/// Session data stored in Redis (mirrors the fields on [`LocalUser`]).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub email: String,
    pub full_name: String,
    pub roles: Vec<String>,
}

const SESSION_PREFIX: &str = "elupane:google:session:";
const SESSION_TTL_SECS: u64 = 7 * 24 * 60 * 60; // 7 days

fn session_key(token: &str) -> String {
    format!("{SESSION_PREFIX}{token}")
}

/// Create a new session and return the session token.
pub async fn create_session(user: &LocalUser) -> Result<String> {
    let token = generate_session_token();
    let session_data = SessionData {
        user_id: user.id.clone(),
        email: user.email.clone(),
        full_name: user.full_name.clone(),
        roles: user.roles.clone(),
    };
    let session_json = serde_json::to_string(&session_data)?;
    cache::session::set(&session_key(&token), &session_json, SESSION_TTL_SECS).await?;
    Ok(token)
}

/// Get session data from a token.
pub async fn get_session(token: &str) -> Result<Option<SessionData>> {
    let Some(session_json) = cache::session::get(&session_key(token)).await? else {
        return Ok(None);
    };
    Ok(Some(serde_json::from_str(&session_json)?))
}

/// Delete a session.
pub async fn delete_session(token: &str) -> Result<()> {
    cache::session::delete(&session_key(token)).await
}

/// Refresh session TTL. Returns `false` if the session did not exist.
pub async fn refresh_session(token: &str) -> Result<bool> {
    let Some(session_json) = cache::session::get(&session_key(token)).await? else {
        return Ok(false);
    };
    cache::session::set(&session_key(token), &session_json, SESSION_TTL_SECS).await?;
    Ok(true)
}

/// Generate a random session token (UUIDv4 - no extra dependencies needed).
fn generate_session_token() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_session_token_is_unique() {
        let t1 = generate_session_token();
        let t2 = generate_session_token();
        assert_ne!(t1, t2);
    }
}