use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GradeCategory {
    pub id: Uuid,
    pub course_id: Uuid,
    pub name: String,
    pub weight: f64,
    pub sort_order: i32,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GradeScale {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub name: String,
    pub is_default: bool,
    pub passing_grade: f64,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GradeScaleBand {
    pub id: Uuid,
    pub scale_id: Uuid,
    pub letter: String,
    pub min_pct: f64,
    pub max_pct: f64,
    pub grade_point: f64,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GradeEntry {
    pub id: Uuid,
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub category_id: Uuid,
    pub source_type: String,
    pub source_id: Uuid,
    pub score: Option<f64>,
    pub max_score: f64,
    pub letter: Option<String>,
    pub grade_point: Option<f64>,
    pub graded_by: Uuid,
    pub graded_at: DateTime<Utc>,
    pub overridden_by: Option<Uuid>,
    pub overridden_at: Option<DateTime<Utc>>,
    pub override_reason: Option<String>,
    pub comments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GradeHistory {
    pub id: Uuid,
    pub grade_entry_id: Uuid,
    pub old_score: Option<f64>,
    pub new_score: Option<f64>,
    pub old_letter: Option<String>,
    pub new_letter: Option<String>,
    pub old_grade_point: Option<f64>,
    pub new_grade_point: Option<f64>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FinalGrade {
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub final_score: Option<f64>,
    pub final_letter: Option<String>,
    pub final_grade_point: Option<f64>,
    pub status: String,
    pub calculated_at: DateTime<Utc>,
    pub calculated_by: Option<Uuid>,
}
