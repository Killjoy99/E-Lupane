//! Authorization: JWT verification and admin/manager gating for the academics
//! plugin. Tokens are issued by the auth plugin via the shared HS256 secret.

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
    #[allow(dead_code)]
    pub fn user_id(&self) -> Result<Uuid> {
        Uuid::parse_str(&self.sub).map_err(|_| anyhow!("token subject is not a valid user id"))
    }

    /// Whether the caller is an administrator.
    #[allow(dead_code)]
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin")
    }

    /// Whether the caller holds a management role (admin or registrar/manager).
    pub fn is_manager(&self) -> bool {
        self.roles
            .iter()
            .any(|r| r == "admin" || r == "registrar" || r == "manager")
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

/// Require the caller to hold a management role (admin / registrar / manager).
pub fn require_manager(claims: &Claims) -> Result<()> {
    if claims.is_manager() {
        Ok(())
    } else {
        Err(anyhow!("forbidden: admin, registrar or manager role required"))
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
        let token = make_token(id, "registrar", &["registrar"]);
        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.user_id().unwrap(), id);
        assert_eq!(claims.username, "registrar");
    }

    #[test]
    fn rejects_garbage_token() {
        assert!(verify_token("nope").is_err());
    }

    #[test]
    fn manager_gating() {
        let admin = verify_token(&make_token(Uuid::new_v4(), "root", &["admin"])).unwrap();
        let registrar = verify_token(&make_token(Uuid::new_v4(), "r", &["registrar"])).unwrap();
        let student = verify_token(&make_token(Uuid::new_v4(), "s", &["student"])).unwrap();
        assert!(require_manager(&admin).is_ok());
        assert!(require_manager(&registrar).is_ok());
        assert!(require_manager(&student).is_err());
    }
}
