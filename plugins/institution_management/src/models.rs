//! Database row models.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct Institution {
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

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct Campus {
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
