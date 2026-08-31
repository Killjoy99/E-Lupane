use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGradeCategoryRequest {
    pub course_id: Uuid,
    pub name: String,
    pub weight: f64,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategoryRequest {
    pub id: Uuid,
    pub name: Option<String>,
    pub weight: Option<f64>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseIdRequest {
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionIdRequest {
    pub institution_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentCourseRequest {
    pub student_id: Uuid,
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdRequest {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGradeScaleRequest {
    pub institution_id: Uuid,
    pub name: String,
    pub passing_grade: f64,
    pub bands: Vec<CreateGradeScaleBandRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGradeScaleBandRequest {
    pub letter: String,
    pub min_pct: f64,
    pub max_pct: f64,
    pub grade_point: f64,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGradeEntryRequest {
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub category_id: Uuid,
    pub source_type: String,
    pub source_id: Uuid,
    pub score: Option<f64>,
    pub max_score: f64,
    pub comments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideGradeEntryRequest {
    pub id: Uuid,
    pub score: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeEntryResponse {
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
    pub is_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalGradeResponse {
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub final_score: Option<f64>,
    pub final_letter: Option<String>,
    pub final_grade_point: Option<f64>,
    pub status: String,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentGradeView {
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub final_grade: Option<FinalGradeResponse>,
    pub entries: Vec<GradeEntryResponse>,
    pub categories: Vec<CategorySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category_id: Uuid,
    pub name: String,
    pub weight: f64,
    pub average: Option<f64>,
}
