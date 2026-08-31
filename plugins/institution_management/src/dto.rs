//! Request/response payloads exchanged over the message envelope.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Institutions
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateInstitutionRequest {
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub short_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub logo_url: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub contact_email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInstitutionRequest {
    pub institution_id: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub short_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub logo_url: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub contact_email: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InstitutionIdRequest {
    pub institution_id: Uuid,
}

// ---------------------------------------------------------------------------
// Campuses
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateCampusRequest {
    pub institution_id: Uuid,
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCampusRequest {
    pub campus_id: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CampusIdRequest {
    pub campus_id: Uuid,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListCampusesRequest {
    #[serde(default)]
    pub institution_id: Option<Uuid>,
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct InstitutionView {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub short_name: String,
    pub description: String,
    pub logo_url: String,
    pub website: String,
    pub contact_email: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct InstitutionList {
    pub institutions: Vec<InstitutionView>,
}

#[derive(Debug, Serialize)]
pub struct CampusView {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub name: String,
    pub code: String,
    pub city: String,
    pub country: String,
    pub address: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CampusList {
    pub campuses: Vec<CampusView>,
}
