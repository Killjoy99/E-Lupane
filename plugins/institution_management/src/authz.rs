//! Authorization: JWT verification and admin gating for institution management.
//!
//! Tokens are issued by the auth plugin (`com.elupane.auth`) and shared via the
//! same HS256 secret (`AUTH_JWT_SECRET`). Institution/campus records are
//! platform-level data, so writes require the `admin` role; any authenticated
//! user may read.

use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// Require the caller to hold the admin role.
pub fn require_admin(claims: &Claims) -> Result<()> {
    if claims.is_admin() {
        Ok(())
    } else {
        Err(anyhow!("forbidden: admin role required"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use jsonwebtoken::{encode, EncodingKey, Header};

    fn make_token(user_id: Uuid, username: &str, roles: &[&str]) -> String {
        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret().as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn token_round_trip() {
        let id = Uuid::new_v4();
        let token = make_token(id, "admin", &["admin"]);
        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.user_id().unwrap(), id);
        assert_eq!(claims.username, "admin");
    }

    #[test]
    fn rejects_garbage_token() {
        assert!(verify_token("not-a-real-token").is_err());
    }

    #[test]
    fn admin_gating() {
        let admin = verify_token(&make_token(Uuid::new_v4(), "root", &["admin"])).unwrap();
        let staff = verify_token(&make_token(Uuid::new_v4(), "staff", &["lecturer"])).unwrap();
        assert!(require_admin(&admin).is_ok());
        assert!(require_admin(&staff).is_err());
    }
}
