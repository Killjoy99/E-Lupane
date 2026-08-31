//! Database row models.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct Course {
    pub id: Uuid,
    pub code: String,
    pub title: String,
    pub short_description: String,
    pub description: String,
    pub thumbnail: String,
    pub banner: String,
    pub category: String,
    pub subject: String,
    pub level: String,
    pub language: String,
    pub difficulty: String,
    pub duration: String,
    pub learning_objectives: Value,
    pub target_audience: Value,
    pub prerequisites: Value,
    pub tags: Value,
    pub status: String,
    pub owner_id: Uuid,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
