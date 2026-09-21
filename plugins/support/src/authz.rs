//! Authorization helpers for the support plugin.
//!
//! The support plugin trusts JWTs issued by the auth plugin (`com.elupane.auth`);
//! both share the same HS256 secret (`AUTH_JWT_SECRET`).

use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

/// Roles treated as "support staff" (can see all tickets, author FAQ).
const STAFF_ROLES: &[&str] = &["admin", "super_admin", "support", "lecturer"];

/// Claims carried in a token issued by the auth plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

impl Claims {
    /// The authenticated user id (the JWT `sub`).
    pub fn user_id(&self) -> Result<uuid::Uuid> {
        uuid::Uuid::parse_str(&self.sub).map_err(|_| anyhow!("token subject is not a valid user id"))
    }

    /// Whether the caller is a platform administrator.
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin" || r == "super_admin")
    }

    /// Whether the caller counts as support staff.
    pub fn is_staff(&self) -> bool {
        self.roles.iter().any(|r| STAFF_ROLES.contains(&r.as_str()))
    }
}

/// Shared HS256 secret (must match the auth plugin's `AUTH_JWT_SECRET`).
fn jwt_secret() -> String {
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

/// Require the caller to be support staff.
pub fn require_staff(claims: &Claims) -> Result<()> {
    if claims.is_staff() {
        Ok(())
    } else {
        Err(anyhow!("forbidden: requires support staff role"))
    }
}
