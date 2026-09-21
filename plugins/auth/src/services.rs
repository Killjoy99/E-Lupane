//! Business logic: user lifecycle, authentication and role assignment.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{BulkCreateError, BulkCreateUserRequest, BulkCreateUserResponse, CreateUserRequest, GoogleLoginRequest, LoginResponse, UserView, VerifyResponse};
use crate::models::User;
use crate::repository;

/// Valid account types (roles) that can be assigned to users when creating accounts.
pub const ACCOUNT_TYPES: &[&str] = &["student", "lecturer", "hod", "dean", "developer", "admin"];

const SEED_ROLE_ORDER: [&str; 6] = ["student", "lecturer", "hod", "dean", "developer", "admin"];

/// JWT claims issued by this plugin.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

impl Claims {
    /// Check if the caller has admin role (or the owner-tier super_admin).
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin" || r == "super_admin")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedAccountSpec {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub role: String,
}

pub fn seeded_account_plan() -> Vec<SeedAccountSpec> {
    let mut out = Vec::new();
    for role in SEED_ROLE_ORDER {
        for idx in 1..=3 {
            let (label, password) = match role {
                "student" => ("Student", "Student123!"),
                "lecturer" => ("Lecturer", "Lecturer123!"),
                "hod" => ("HoD", "Hod123!"),
                "dean" => ("Dean", "Dean123!"),
                "developer" => ("Developer", "Developer123!"),
                "admin" => ("Admin", "Admin123!"),
                other => unreachable!("unsupported seed role {other}"),
            };

            out.push(SeedAccountSpec {
                username: format!("{role}{idx}"),
                email: format!("{role}{idx}@elupane.local"),
                password: password.to_string(),
                full_name: format!("{label} {idx}"),
                role: role.to_string(),
            });
        }
    }
    out
}

pub async fn ensure_seed_accounts(pool: &PgPool) -> Result<Vec<UserView>> {
    let mut created = Vec::new();
    for seed in seeded_account_plan() {
        if repository::find_user_by_username(pool, &seed.username).await?.is_some() {
            continue;
        }
        let req = CreateUserRequest {
            username: seed.username,
            email: seed.email,
            password: seed.password,
            full_name: seed.full_name,
            roles: vec![seed.role],
            admin_token: None,
        };
        created.push(create_user(pool, req).await?);
    }
    Ok(created)
}

pub async fn create_user(pool: &PgPool, req: CreateUserRequest) -> Result<UserView> {
    if repository::find_user_by_username(pool, &req.username).await?.is_some() {
        return Err(anyhow!("username already exists"));
    }
    if repository::find_user_by_email(pool, &req.email).await?.is_some() {
        return Err(anyhow!("email already exists"));
    }

    let password_hash = bcrypt::hash(&req.password, 12)?;
    let user = repository::create_user(pool, &req.username, &req.email, &password_hash, &req.full_name)
        .await?;

    let roles = if req.roles.is_empty() {
        vec!["student".to_string()]
    } else {
        req.roles
    };
    for role in &roles {
        if let Some(r) = repository::find_role_by_name(pool, role).await? {
            repository::assign_role(pool, user.id, r.id).await?;
        }
    }

    to_view(pool, &user).await
}

pub async fn bulk_create_users(pool: &PgPool, req: BulkCreateUserRequest) -> Result<BulkCreateUserResponse> {
    let mut created = Vec::new();
    let mut errors = Vec::new();

    for (idx, user_req) in req.users.iter().enumerate() {
        match create_user(pool, user_req.clone()).await {
            Ok(view) => created.push(view),
            Err(err) => errors.push(BulkCreateError {
                index: idx,
                username: user_req.username.clone(),
                message: err.to_string(),
            }),
        }
    }

    Ok(BulkCreateUserResponse { created, errors })
}

pub async fn delete_user(pool: &PgPool, user_id: Uuid) -> Result<()> {
    repository::delete_user(pool, user_id).await?;
    Ok(())
}

pub async fn get_user(pool: &PgPool, user_id: Uuid) -> Result<UserView> {
    let user = repository::find_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| anyhow!("user not found"))?;
    to_view(pool, &user).await
}

pub async fn list_users(pool: &PgPool) -> Result<Vec<UserView>> {
    let users = repository::list_users(pool).await?;
    let mut views = Vec::new();
    for user in users {
        views.push(to_view(pool, &user).await?);
    }
    Ok(views)
}

async fn to_view(pool: &PgPool, user: &User) -> Result<UserView> {
    let roles = repository::roles_for_user(pool, user.id).await?;
    Ok(to_view_from_parts(user.clone(), roles))
}

fn to_view_from_parts(user: User, roles: Vec<String>) -> UserView {
    UserView {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        active: user.active,
        roles,
    }
}

pub async fn authenticate(pool: &PgPool, username: &str, password: &str) -> Result<LoginResponse> {
    let user = repository::find_user_by_username(pool, username)
        .await?
        .ok_or_else(|| anyhow!("invalid username or password"))?;

    if !user.active {
        return Err(anyhow!("user is disabled"));
    }

    if !bcrypt::verify(password, &user.password_hash)? {
        return Err(anyhow!("invalid username or password"));
    }

    let roles = repository::roles_for_user(pool, user.id).await?;
    let token = issue_token(&user.id, &user.username, &roles)?;

    Ok(LoginResponse {
        token,
        user: to_view_from_parts(user, roles),
    })
}

pub async fn google_login(pool: &PgPool, req: GoogleLoginRequest) -> Result<LoginResponse> {
    let email = req.email.trim().to_ascii_lowercase();
    let roles = if req.roles.is_empty() {
        vec!["student".to_string()]
    } else {
        req.roles
    };

    if let Some(existing) = repository::find_user_by_email(pool, &email).await? {
        let current_roles = repository::roles_for_user(pool, existing.id).await?;
        for role_name in &current_roles {
            if let Some(r) = repository::find_role_by_name(pool, role_name).await? {
                repository::revoke_role(pool, existing.id, r.id).await?;
            }
        }
        for role_name in &roles {
            if let Some(r) = repository::find_role_by_name(pool, role_name).await? {
                repository::assign_role(pool, existing.id, r.id).await?;
            }
        }
        let updated_roles = repository::roles_for_user(pool, existing.id).await?;
        let token = issue_token(&existing.id, &existing.username, &updated_roles)?;
        return Ok(LoginResponse {
            token,
            user: to_view_from_parts(existing, updated_roles),
        });
    }

    let username = email.clone();
    let full_name = if req.full_name.trim().is_empty() {
        email.clone()
    } else {
        req.full_name.trim().to_string()
    };
    let password = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash(&password, 12)?;

    let user = repository::create_user(pool, &username, &email, &password_hash, &full_name).await?;

    for role in &roles {
        if let Some(r) = repository::find_role_by_name(pool, role).await? {
            repository::assign_role(pool, user.id, r.id).await?;
        }
    }

    let user_roles = repository::roles_for_user(pool, user.id).await?;
    let token = issue_token(&user.id, &user.username, &user_roles)?;

    Ok(LoginResponse {
        token,
        user: to_view_from_parts(user, user_roles),
    })
}

pub async fn verify(pool: &PgPool, token: &str) -> Result<VerifyResponse> {
    match verify_token(token) {
        Ok(claims) => {
            let user_id = Uuid::parse_str(&claims.sub)
                .map_err(|_| anyhow!("token subject is not a valid user id"))?;
            match repository::find_user_by_id(pool, user_id).await? {
                Some(user) => {
                    let roles = repository::roles_for_user(pool, user.id).await?;
                    Ok(VerifyResponse {
                        valid: true,
                        user: Some(to_view_from_parts(user, roles)),
                        reason: None,
                    })
                }
                None => Ok(VerifyResponse {
                    valid: false,
                    user: None,
                    reason: Some("user not found".to_string()),
                }),
            }
        }
        Err(err) => Ok(VerifyResponse {
            valid: false,
            user: None,
            reason: Some(err.to_string()),
        }),
    }
}

pub async fn assign_role(pool: &PgPool, user_id: Uuid, role_name: &str) -> Result<UserView> {
    let user = repository::find_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| anyhow!("user not found"))?;
    let role = repository::find_role_by_name(pool, role_name)
        .await?
        .ok_or_else(|| anyhow!("role not found: {role_name}"))?;
    repository::assign_role(pool, user.id, role.id).await?;
    to_view(pool, &user).await
}

pub async fn list_roles(pool: &PgPool) -> Result<Vec<serde_json::Value>> {
    let roles = repository::list_roles(pool).await?;
    let mut out = Vec::new();
    for role in roles {
        let permissions = repository::permissions_for_role(pool, role.id).await?;
        out.push(serde_json::json!({
            "id": role.id,
            "name": role.name,
            "description": role.description,
            "permissions": permissions,
        }));
    }
    Ok(out)
}

pub async fn list_permissions(pool: &PgPool) -> Result<Vec<serde_json::Value>> {
    let permissions = repository::list_permissions(pool).await?;
    Ok(permissions
        .into_iter()
        .map(|p| serde_json::json!({ "id": p.id, "name": p.name, "description": p.description }))
        .collect())
}

pub async fn revoke_role(pool: &PgPool, user_id: Uuid, role_name: &str) -> Result<UserView> {
    let user = repository::find_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| anyhow!("user not found"))?;
    let role = repository::find_role_by_name(pool, role_name)
        .await?
        .ok_or_else(|| anyhow!("role not found: {role_name}"))?;
    repository::revoke_role(pool, user.id, role.id).await?;
    to_view(pool, &user).await
}

/// Replace a role's permission set (the permission-matrix "save row" operation).
pub async fn set_role_permissions(
    pool: &PgPool,
    role_name: &str,
    permissions: Vec<String>,
) -> Result<serde_json::Value> {
    let role = repository::find_role_by_name(pool, role_name)
        .await?
        .ok_or_else(|| anyhow!("role not found: {role_name}"))?;
    repository::set_role_permissions(pool, role.id, &permissions).await?;
    let granted = repository::permissions_for_role(pool, role.id).await?;
    Ok(serde_json::json!({
        "id": role.id,
        "name": role.name,
        "description": role.description,
        "permissions": granted,
    }))
}

/// Create (or update the description of) a role, optionally seeding permissions.
pub async fn create_role(
    pool: &PgPool,
    name: &str,
    description: Option<String>,
    permissions: Vec<String>,
) -> Result<serde_json::Value> {
    let role = repository::create_role(pool, name, description.as_deref()).await?;
    repository::set_role_permissions(pool, role.id, &permissions).await?;
    let granted = repository::permissions_for_role(pool, role.id).await?;
    Ok(serde_json::json!({
        "id": role.id,
        "name": role.name,
        "description": role.description,
        "permissions": granted,
    }))
}

fn jwt_secret() -> String {
    std::env::var("AUTH_JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string())
}

/// Issue a signed JWT access token (HS256), valid for 12 hours.
pub fn issue_token(user_id: &Uuid, username: &str, roles: &[String]) -> Result<String> {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(12)).timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        roles: roles.to_vec(),
        exp,
    };
    Ok(jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret().as_bytes()),
    )?)
}

/// Validate a JWT and return its claims.
pub fn verify_token(token: &str) -> Result<Claims> {
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret().as_bytes()),
        &jsonwebtoken::Validation::default(),
    )?;
    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_account_plan_has_three_accounts_per_role() {
        let plan = seeded_account_plan();
        assert_eq!(plan.len(), 18);

        let mut counts = std::collections::BTreeMap::new();
        for acc in &plan {
            *counts.entry(acc.role.clone()).or_insert(0usize) += 1;
        }

        for role in ["student", "lecturer", "hod", "dean", "developer", "admin"] {
            assert_eq!(counts.get(role), Some(&3), "missing three accounts for role {role}");
        }

        assert!(plan.iter().all(|acc| !acc.username.is_empty()));
        assert!(plan.iter().all(|acc| acc.email.ends_with("@elupane.local")));
    }

    #[test]
    fn jwt_round_trip() {
        let id = Uuid::new_v4();
        let token = issue_token(&id, "alice", &["student".to_string()]).unwrap();

        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.sub, id.to_string());
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.roles, vec!["student".to_string()]);
    }

    #[test]
    fn jwt_rejects_bad_token() {
        assert!(verify_token("not-a-token").is_err());
    }

    #[test]
    fn bcrypt_hash_and_verify() {
        let hash = bcrypt::hash("s3cret", 4).unwrap();
        assert!(bcrypt::verify("s3cret", &hash).unwrap());
        assert!(!bcrypt::verify("wrong", &hash).unwrap());
    }
}
