use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AnalyticsEvent {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: Uuid,
    pub course_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AnalyticsAggregate {
    pub id: Uuid,
    pub bucket: String,
    pub bucket_start: DateTime<Utc>,
    pub bucket_end: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub metric: String,
    pub value: f64,
    pub metadata: serde_json::Value,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AnalyticsReport {
    pub id: Uuid,
    pub report_type: String,
    pub format: String,
    pub filters: serde_json::Value,
    pub data: serde_json::Value,
    pub file_path: Option<String>,
    pub generated_by: Uuid,
    pub generated_at: DateTime<Utc>,
}
