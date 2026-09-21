//! Authorization helpers for search plugin.

use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

/// Claims extracted from a JWT token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub admin: bool,
    pub exp: usize,
}

impl Claims {
    /// Get the user ID from the subject claim.
    pub fn user_id(&self) -> Result<uuid::Uuid> {
        uuid::Uuid::parse_str(&self.sub).map_err(|e| anyhow!("invalid user_id: {e}"))
    }

    /// Check if the user has admin privileges.
    pub fn is_admin(&self) -> bool {
        self.admin
    }
}

/// Verify a JWT token and extract claims.
pub fn verify_token(token: &str) -> Result<Claims> {
    let secret = std::env::var("AUTH_JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());

    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}
