use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ADMIN_ROLES: &[&str] = &["admin"];
pub const INSTRUCTOR_ROLES: &[&str] = &["admin", "lecturer"];

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
        self.roles.iter().any(|r| ADMIN_ROLES.contains(&r.as_str()))
    }

    pub fn is_instructor(&self) -> bool {
        self.roles.iter().any(|r| INSTRUCTOR_ROLES.contains(&r.as_str()))
    }
}

pub fn jwt_secret() -> String {
    std::env::var("AUTH_JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string())
}

pub fn verify_token(token: &str) -> Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

pub fn require_admin(claims: &Claims) -> Result<()> {
    if claims.is_admin() {
        Ok(())
    } else {
        Err(anyhow!("forbidden: requires admin role"))
    }
}

pub fn require_instructor(claims: &Claims) -> Result<()> {
    if claims.is_instructor() {
        Ok(())
    } else {
        Err(anyhow!("forbidden: requires instructor role"))
    }
}
