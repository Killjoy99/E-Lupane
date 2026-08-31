//! Authorization: JWT verification shared with the auth plugin.
//!
//! Live-session permission model:
//! * **Instructors/admins** may schedule, start, end, cancel a session for a
//!   course they manage, and mark attendance / attach resources.
//! * **Students** may join a session for a course they are actively enrolled
//!   in (or any course before enrollment is wired up, mirroring the
//!   enrollments plugin's defer-until-initialized behaviour).
//! * Chat history is visible to anyone who could join the session.

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
    pub fn user_id(&self) -> Result<Uuid> {
        Uuid::parse_str(&self.sub).map_err(|_| anyhow!("token subject is not a valid user id"))
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use jsonwebtoken::{encode, EncodingKey, Header};

    fn make_token(user_id: Uuid, roles: &[&str]) -> String {
        let claims = Claims {
            sub: user_id.to_string(),
            username: "user".into(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        };
        encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret().as_bytes())).unwrap()
    }

    #[test]
    fn token_round_trip() {
        let id = Uuid::new_v4();
        let c = verify_token(&make_token(id, &["lecturer"])).unwrap();
        assert_eq!(c.user_id().unwrap(), id);
        assert!(!c.is_admin());
    }
}
