//! Authorization: JWT verification and role checks.
//!
//! The assignments plugin trusts tokens issued by the auth plugin
//! (`com.elupane.auth`). Both plugins share the same HS256 secret
//! (`AUTH_JWT_SECRET`), so the caller's identity and roles can be verified
//! locally without an extra round-trip over the message bus.

use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Roles allowed to manage assignments and grade submissions.
pub const INSTRUCTOR_ROLES: &[&str] = &["admin", "lecturer"];
/// Roles allowed to submit work as a student.
pub const STUDENT_ROLES: &[&str] = &["student", "admin"];

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

    /// Whether the caller may act as a student.
    pub fn is_student(&self) -> bool {
        self.roles.iter().any(|r| STUDENT_ROLES.contains(&r.as_str()))
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

/// Require a student or admin.
pub fn require_student(claims: &Claims) -> Result<()> {
    require_role(claims, STUDENT_ROLES)
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
        let token = make_token(id, "alice", &["student"]);
        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.user_id().unwrap(), id);
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.roles, vec!["student"]);
    }

    #[test]
    fn rejects_garbage_token() {
        assert!(verify_token("not-a-real-token").is_err());
    }

    #[test]
    fn role_classification() {
        let lecturer = make_token(Uuid::new_v4(), "dr", &["lecturer"]);
        let admin = make_token(Uuid::new_v4(), "root", &["admin"]);
        let student = make_token(Uuid::new_v4(), "bob", &["student"]);

        assert!(verify_token(&lecturer).unwrap().is_instructor());
        assert!(verify_token(&admin).unwrap().is_instructor());
        assert!(!verify_token(&student).unwrap().is_instructor());

        assert!(verify_token(&student).unwrap().is_student());
        assert!(verify_token(&admin).unwrap().is_student());
        assert!(!verify_token(&lecturer).unwrap().is_student());
    }

    #[test]
    fn instructor_checks_enforced() {
        let lecturer = verify_token(&make_token(Uuid::new_v4(), "dr", &["lecturer"])).unwrap();
        let student = verify_token(&make_token(Uuid::new_v4(), "bob", &["student"])).unwrap();

        assert!(require_instructor(&lecturer).is_ok());
        assert!(require_instructor(&student).is_err());

        assert!(require_student(&student).is_ok());
        assert!(require_student(&lecturer).is_err());
    }
}
