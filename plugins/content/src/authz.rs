//! Authorization: JWT verification and content ownership rules.
//!
//! The content plugin trusts tokens issued by the auth plugin
//! (`com.elupane.auth`). Both plugins share the same HS256 secret
//! (`AUTH_JWT_SECRET`), so identity + roles are verified locally.
//!
//! Ownership model:
//! * Admins may do anything.
//! * Instructors who manage the parent course may author content.
//! * Students may view published content only.

use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Roles allowed to create and manage content.
pub const INSTRUCTOR_ROLES: &[&str] = &["admin", "lecturer"];

/// The claims carried in a token issued by the auth plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

impl Claims {
    /// The authenticated user id (the JWT `sub`).
    pub fn user_id(&self) -> Result<Uuid> {
        Uuid::parse_str(&self.sub).map_err(|_| anyhow!("token subject is not a valid user id"))
    }

    /// Whether the caller is an administrator.
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin")
    }

    /// Whether the caller holds an instructor role (admin or lecturer).
    pub fn is_instructor(&self) -> bool {
        self.roles.iter().any(|r| INSTRUCTOR_ROLES.contains(&r.as_str()))
    }
}

/// Shared HS256 secret (must match the auth plugin's `AUTH_JWT_SECRET`).
pub fn jwt_secret() -> String {
    std::env::var("AUTH_JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string())
}

/// Verify a JWT and return its claims.
pub fn verify_token(token: &str) -> Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

/// Require the caller to hold at least one of the given roles.
pub fn require_role(claims: &Claims, allowed: &[&str]) -> Result<()> {
    if claims.roles.iter().any(|r| allowed.contains(&r.as_str())) {
        Ok(())
    } else {
        Err(anyhow!("forbidden: requires one of {allowed:?}"))
    }
}

/// Require an instructor (admin or lecturer).
pub fn require_instructor(claims: &Claims) -> Result<()> {
    require_role(claims, INSTRUCTOR_ROLES)
}
