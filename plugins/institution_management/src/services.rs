//! Business logic: institution and campus lifecycle with validation and
//! admin-gated authorization. Pure validation helpers are unit-tested.

use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authz::{self, Claims};
use crate::dto::{
    CampusList, CampusView, CreateCampusRequest, CreateInstitutionRequest, InstitutionList,
    InstitutionView, UpdateCampusRequest, UpdateInstitutionRequest,
};
use crate::repository;

pub const MAX_CODE_LEN: usize = 20;
pub const MAX_NAME_LEN: usize = 200;

/// Validate a short identifier (institution/campus code): alphanumerics plus
/// dashes/underscores, upper-case, non-empty.
pub fn validate_code(code: &str, what: &str) -> Result<()> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("{what} code is required"));
    }
    if trimmed.len() > MAX_CODE_LEN {
        return Err(anyhow!("{what} code too long (max {MAX_CODE_LEN} characters)"));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "{what} code may only contain letters, digits, dashes and underscores"
        ));
    }
    if trimmed.chars().any(|c| c.is_ascii_alphabetic()) && trimmed != trimmed.to_uppercase() {
        return Err(anyhow!("{what} code must be upper-case"));
    }
    Ok(())
}

/// Validate a human-readable name.
pub fn validate_name(name: &str, what: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("{what} name is required"));
    }
    if trimmed.len() > MAX_NAME_LEN {
        return Err(anyhow!("{what} name too long (max {MAX_NAME_LEN} characters)"));
    }
    Ok(())
}

/// Normalize a status string; defaults to `active`, otherwise must be one of
/// the known values.
pub fn normalize_status(s: &str) -> Result<&'static str> {
    match s {
        "active" => Ok("active"),
        "inactive" => Ok("inactive"),
        "" => Ok("active"),
        other => Err(anyhow!("invalid status '{other}' (expected 'active' or 'inactive')")),
    }
}

fn institution_view(i: &crate::models::Institution) -> InstitutionView {
    InstitutionView {
        id: i.id,
        name: i.name.clone(),
        code: i.code.clone(),
        short_name: i.short_name.clone(),
        description: i.description.clone(),
        logo_url: i.logo_url.clone(),
        website: i.website.clone(),
        contact_email: i.contact_email.clone(),
        status: i.status.clone(),
        created_at: i.created_at,
        updated_at: i.updated_at,
    }
}

fn campus_view(c: &crate::models::Campus) -> CampusView {
    CampusView {
        id: c.id,
        institution_id: c.institution_id,
        name: c.name.clone(),
        code: c.code.clone(),
        city: c.city.clone(),
        country: c.country.clone(),
        address: c.address.clone(),
        status: c.status.clone(),
        created_at: c.created_at,
        updated_at: c.updated_at,
    }
}

// ---------------------------------------------------------------------------
// Institutions
// ---------------------------------------------------------------------------

pub async fn create_institution(
    pool: &PgPool,
    claims: &Claims,
    req: CreateInstitutionRequest,
) -> Result<InstitutionView> {
    authz::require_admin(claims)?;
    validate_name(&req.name, "institution")?;
    validate_code(&req.code, "institution")?;
    if repository::find_institution_by_code(pool, &req.code).await?.is_some() {
        return Err(anyhow!("an institution with code '{}' already exists", req.code));
    }
    let row = repository::insert_institution(
        pool,
        req.name.trim(),
        req.code.trim().to_uppercase().as_str(),
        req.short_name.trim(),
        req.description.trim(),
        req.logo_url.trim(),
        req.website.trim(),
        req.contact_email.trim(),
    )
    .await?;
    Ok(institution_view(&row))
}

pub async fn get_institution(pool: &PgPool, id: Uuid) -> Result<InstitutionView> {
    let row = repository::find_institution_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow!("institution not found"))?;
    Ok(institution_view(&row))
}

pub async fn list_institutions(pool: &PgPool) -> Result<InstitutionList> {
    let rows = repository::list_institutions(pool).await?;
    Ok(InstitutionList {
        institutions: rows.iter().map(institution_view).collect(),
    })
}

pub async fn update_institution(
    pool: &PgPool,
    claims: &Claims,
    req: UpdateInstitutionRequest,
) -> Result<InstitutionView> {
    authz::require_admin(claims)?;
    let existing = repository::find_institution_by_id(pool, req.institution_id)
        .await?
        .ok_or_else(|| anyhow!("institution not found"))?;

    let name = req.name.unwrap_or(existing.name.clone());
    validate_name(&name, "institution")?;

    let code = req.code.unwrap_or(existing.code.clone());
    validate_code(&code, "institution")?;
    if let Some(dup) = repository::find_institution_by_code(pool, &code).await? {
        if dup.id != req.institution_id {
            return Err(anyhow!("an institution with code '{code}' already exists"));
        }
    }

    if let Some(status) = &req.status {
        normalize_status(status)?;
    }

    let row = repository::update_institution(
        pool,
        req.institution_id,
        Some(name.trim()),
        Some(code.trim().to_uppercase().as_str()),
        req.short_name.as_deref().map(str::trim),
        req.description.as_deref().map(str::trim),
        req.logo_url.as_deref().map(str::trim),
        req.website.as_deref().map(str::trim),
        req.contact_email.as_deref().map(str::trim),
        req.status.as_deref(),
    )
    .await?;
    Ok(institution_view(&row))
}

pub async fn delete_institution(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<()> {
    authz::require_admin(claims)?;
    if !repository::delete_institution(pool, id).await? {
        return Err(anyhow!("institution not found"));
    }
    Ok(())
}


// ---------------------------------------------------------------------------
// Campuses
// ---------------------------------------------------------------------------

pub async fn create_campus(
    pool: &PgPool,
    claims: &Claims,
    req: CreateCampusRequest,
) -> Result<CampusView> {
    authz::require_admin(claims)?;
    if repository::find_institution_by_id(pool, req.institution_id).await?.is_none() {
        return Err(anyhow!("institution not found"));
    }
    validate_name(&req.name, "campus")?;
    validate_code(&req.code, "campus")?;
    if repository::find_campus_by_code(pool, &req.code).await?.is_some() {
        return Err(anyhow!("a campus with code '{}' already exists", req.code));
    }
    let row = repository::insert_campus(
        pool,
        req.institution_id,
        req.name.trim(),
        req.code.trim().to_uppercase().as_str(),
        req.city.trim(),
        req.country.trim(),
        req.address.trim(),
    )
    .await?;
    Ok(campus_view(&row))
}

pub async fn get_campus(pool: &PgPool, id: Uuid) -> Result<CampusView> {
    let row = repository::find_campus_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow!("campus not found"))?;
    Ok(campus_view(&row))
}

pub async fn list_campuses(pool: &PgPool, institution_id: Option<Uuid>) -> Result<CampusList> {
    let rows = repository::list_campuses(pool, institution_id).await?;
    Ok(CampusList {
        campuses: rows.iter().map(campus_view).collect(),
    })
}

pub async fn update_campus(
    pool: &PgPool,
    claims: &Claims,
    req: UpdateCampusRequest,
) -> Result<CampusView> {
    authz::require_admin(claims)?;
    let existing = repository::find_campus_by_id(pool, req.campus_id)
        .await?
        .ok_or_else(|| anyhow!("campus not found"))?;

    let name = req.name.unwrap_or(existing.name.clone());
    validate_name(&name, "campus")?;

    let code = req.code.unwrap_or(existing.code.clone());
    validate_code(&code, "campus")?;
    if let Some(dup) = repository::find_campus_by_code(pool, &code).await? {
        if dup.id != req.campus_id {
            return Err(anyhow!("a campus with code '{code}' already exists"));
        }
    }

    if let Some(status) = &req.status {
        normalize_status(status)?;
    }

    let row = repository::update_campus(
        pool,
        req.campus_id,
        Some(name.trim()),
        Some(code.trim().to_uppercase().as_str()),
        req.city.as_deref().map(str::trim),
        req.country.as_deref().map(str::trim),
        req.address.as_deref().map(str::trim),
        req.status.as_deref(),
    )
    .await?;
    Ok(campus_view(&row))
}

pub async fn delete_campus(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<()> {
    authz::require_admin(claims)?;
    if !repository::delete_campus(pool, id).await? {
        return Err(anyhow!("campus not found"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests (pure logic — no database)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_validation() {
        assert!(validate_code("UNIV001", "institution").is_ok());
        assert!(validate_code("MAIN_CAMPUS", "campus").is_ok());
        assert!(validate_code("", "campus").is_err());
        assert!(validate_code("   ", "campus").is_err());
        assert!(validate_code("campus one", "campus").is_err()); // spaces
        assert!(validate_code("lower", "campus").is_err()); // not upper-case
        assert!(validate_code("THIS-CODE-IS-WAY-TOO-LONG-12345", "campus").is_err());
    }

    #[test]
    fn name_validation() {
        assert!(validate_name("Great Zimbabwe University", "institution").is_ok());
        assert!(validate_name("", "institution").is_err());
        assert!(validate_name(&"x".repeat(300), "institution").is_err());
    }

    #[test]
    fn status_normalization() {
        assert_eq!(normalize_status("active").unwrap(), "active");
        assert_eq!(normalize_status("inactive").unwrap(), "inactive");
        assert_eq!(normalize_status("").unwrap(), "active");
        assert!(normalize_status("bogus").is_err());
    }
}

