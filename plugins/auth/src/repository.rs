//! SQLx data access for users, roles and permissions.

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Permission, Role, User};

pub async fn find_user_by_username(pool: &PgPool, username: &str) -> Result<Option<User>> {
    Ok(sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, full_name, active, created_at \
         FROM users WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
    Ok(sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, full_name, active, created_at \
         FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>> {
    Ok(sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, full_name, active, created_at \
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password_hash: &str,
    full_name: &str,
) -> Result<User> {
    Ok(sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, password_hash, full_name) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, username, email, password_hash, full_name, active, created_at",
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(full_name)
    .fetch_one(pool)
    .await?)
}

pub async fn count_users(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(pool)
        .await?)
}

pub async fn list_users(pool: &PgPool) -> Result<Vec<User>> {
    Ok(sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, full_name, active, created_at \
         FROM users ORDER BY created_at DESC LIMIT 200",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn roles_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT r.name FROM roles r \
         JOIN user_roles ur ON ur.role_id = r.id \
         WHERE ur.user_id = $1 ORDER BY r.name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn find_role_by_name(pool: &PgPool, name: &str) -> Result<Option<Role>> {
    Ok(sqlx::query_as::<_, Role>(
        "SELECT id, name, description FROM roles WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?)
}

pub async fn assign_role(pool: &PgPool, user_id: Uuid, role_id: Uuid) -> Result<()> {
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .bind(role_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_roles(pool: &PgPool) -> Result<Vec<Role>> {
    Ok(sqlx::query_as::<_, Role>("SELECT id, name, description FROM roles ORDER BY name")
        .fetch_all(pool)
        .await?)
}

pub async fn list_permissions(pool: &PgPool) -> Result<Vec<Permission>> {
    Ok(sqlx::query_as::<_, Permission>(
        "SELECT id, name, description FROM permissions ORDER BY name",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn permissions_for_role(pool: &PgPool, role_id: Uuid) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT p.name FROM permissions p \
         JOIN role_permissions rp ON rp.permission_id = p.id \
         WHERE rp.role_id = $1 ORDER BY p.name",
    )
    .bind(role_id)
    .fetch_all(pool)
    .await?)
}

pub async fn revoke_role(pool: &PgPool, user_id: Uuid, role_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
        .bind(user_id)
        .bind(role_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Replace a role's permission set atomically (delete + insert in one statement).
pub async fn set_role_permissions(pool: &PgPool, role_id: Uuid, permissions: &[String]) -> Result<()> {
    sqlx::query(
        "WITH del AS ( \
            DELETE FROM role_permissions WHERE role_id = $1 \
         ) \
         INSERT INTO role_permissions (role_id, permission_id) \
         SELECT $1, p.id FROM permissions p WHERE p.name = ANY($2::TEXT[]) \
         ON CONFLICT DO NOTHING",
    )
    .bind(role_id)
    .bind(permissions.to_vec())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn create_role(pool: &PgPool, name: &str, description: Option<&str>) -> Result<Role> {
    Ok(sqlx::query_as::<_, Role>(
        "INSERT INTO roles (name, description) VALUES ($1, $2) \
         ON CONFLICT (name) DO UPDATE SET description = EXCLUDED.description \
         RETURNING id, name, description",
    )
    .bind(name)
    .bind(description)
    .fetch_one(pool)
    .await?)
}


pub async fn delete_user(pool: &PgPool, user_id: Uuid) -> Result<()> {
    // Delete user role assignments first
    sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    // Then delete the user
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
