//! SQLx data access for institutions and campuses. All filters are bound
//! parameters (never interpolated); the column list is a compile-time constant
//! wrapped in `AssertSqlSafe`.

use anyhow::Result;
use sqlx::{AssertSqlSafe, PgPool};
use uuid::Uuid;

use crate::models::{Campus, Institution};

pub const INSTITUTION_COLUMNS: &str = "id, name, code, short_name, description, logo_url, \
    website, contact_email, status, created_at, updated_at";

pub const CAMPUS_COLUMNS: &str = "id, institution_id, name, code, city, country, address, \
    status, created_at, updated_at";

// ---------------------------------------------------------------------------
// Institutions
// ---------------------------------------------------------------------------

pub async fn find_institution_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Institution>> {
    Ok(sqlx::query_as::<_, Institution>(AssertSqlSafe(format!(
        "SELECT {INSTITUTION_COLUMNS} FROM institutions WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_institution_by_code(pool: &PgPool, code: &str) -> Result<Option<Institution>> {
    Ok(sqlx::query_as::<_, Institution>(AssertSqlSafe(format!(
        "SELECT {INSTITUTION_COLUMNS} FROM institutions WHERE lower(code) = lower($1)"
    )))
    .bind(code)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_institutions(pool: &PgPool) -> Result<Vec<Institution>> {
    Ok(sqlx::query_as::<_, Institution>(AssertSqlSafe(format!(
        "SELECT {INSTITUTION_COLUMNS} FROM institutions ORDER BY name"
    )))
    .fetch_all(pool)
    .await?)
}

pub async fn insert_institution(
    pool: &PgPool,
    name: &str,
    code: &str,
    short_name: &str,
    description: &str,
    logo_url: &str,
    website: &str,
    contact_email: &str,
) -> Result<Institution> {
    Ok(sqlx::query_as::<_, Institution>(AssertSqlSafe(format!(
        "INSERT INTO institutions \
            (name, code, short_name, description, logo_url, website, contact_email) \
         VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING {INSTITUTION_COLUMNS}"
    )))
    .bind(name)
    .bind(code)
    .bind(short_name)
    .bind(description)
    .bind(logo_url)
    .bind(website)
    .bind(contact_email)
    .fetch_one(pool)
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_institution(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    code: Option<&str>,
    short_name: Option<&str>,
    description: Option<&str>,
    logo_url: Option<&str>,
    website: Option<&str>,
    contact_email: Option<&str>,
    status: Option<&str>,
) -> Result<Institution> {
    Ok(sqlx::query_as::<_, Institution>(AssertSqlSafe(format!(
        "UPDATE institutions SET \
            name = COALESCE($2, name), \
            code = COALESCE($3, code), \
            short_name = COALESCE($4, short_name), \
            description = COALESCE($5, description), \
            logo_url = COALESCE($6, logo_url), \
            website = COALESCE($7, website), \
            contact_email = COALESCE($8, contact_email), \
            status = COALESCE($9, status), \
            updated_at = now() \
         WHERE id = $1 RETURNING {INSTITUTION_COLUMNS}"
    )))
    .bind(id)
    .bind(name)
    .bind(code)
    .bind(short_name)
    .bind(description)
    .bind(logo_url)
    .bind(website)
    .bind(contact_email)
    .bind(status)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_institution(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM institutions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Campuses
// ---------------------------------------------------------------------------

pub async fn find_campus_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Campus>> {
    Ok(sqlx::query_as::<_, Campus>(AssertSqlSafe(format!(
        "SELECT {CAMPUS_COLUMNS} FROM campuses WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_campus_by_code(pool: &PgPool, code: &str) -> Result<Option<Campus>> {
    Ok(sqlx::query_as::<_, Campus>(AssertSqlSafe(format!(
        "SELECT {CAMPUS_COLUMNS} FROM campuses WHERE lower(code) = lower($1)"
    )))
    .bind(code)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_campuses(pool: &PgPool, institution_id: Option<Uuid>) -> Result<Vec<Campus>> {
    Ok(sqlx::query_as::<_, Campus>(AssertSqlSafe(format!(
        "SELECT {CAMPUS_COLUMNS} FROM campuses \
         WHERE ($1::uuid IS NULL OR institution_id = $1) \
         ORDER BY name"
    )))
    .bind(institution_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_campus(
    pool: &PgPool,
    institution_id: Uuid,
    name: &str,
    code: &str,
    city: &str,
    country: &str,
    address: &str,
) -> Result<Campus> {
    Ok(sqlx::query_as::<_, Campus>(AssertSqlSafe(format!(
        "INSERT INTO campuses (institution_id, name, code, city, country, address) \
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING {CAMPUS_COLUMNS}"
    )))
    .bind(institution_id)
    .bind(name)
    .bind(code)
    .bind(city)
    .bind(country)
    .bind(address)
    .fetch_one(pool)
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_campus(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    code: Option<&str>,
    city: Option<&str>,
    country: Option<&str>,
    address: Option<&str>,
    status: Option<&str>,
) -> Result<Campus> {
    Ok(sqlx::query_as::<_, Campus>(AssertSqlSafe(format!(
        "UPDATE campuses SET \
            name = COALESCE($2, name), \
            code = COALESCE($3, code), \
            city = COALESCE($4, city), \
            country = COALESCE($5, country), \
            address = COALESCE($6, address), \
            status = COALESCE($7, status), \
            updated_at = now() \
         WHERE id = $1 RETURNING {CAMPUS_COLUMNS}"
    )))
    .bind(id)
    .bind(name)
    .bind(code)
    .bind(city)
    .bind(country)
    .bind(address)
    .bind(status)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_campus(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM campuses WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

