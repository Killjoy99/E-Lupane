use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseIdRequest {
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentCourseRequest {
    pub student_id: Uuid,
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionIdRequest {
    pub institution_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRangeRequest {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportRequest {
    pub report_type: String,
    pub format: String,
    pub filters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentProgressResponse {
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub completion_rate: f64,
    pub lessons_completed: i64,
    pub total_lessons: i64,
    pub time_spent_minutes: i64,
    pub quiz_average: Option<f64>,
    pub assignment_average: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructorAnalyticsResponse {
    pub instructor_id: Uuid,
    pub course_id: Uuid,
    pub enrollment_count: i64,
    pub avg_completion_rate: f64,
    pub avg_grade: Option<f64>,
    pub engagement_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAnalyticsResponse {
    pub active_users: i64,
    pub new_enrollments: i64,
    pub completion_rate: f64,
    pub dropout_rate: f64,
    pub course_count: i64,
    pub instructor_count: i64,
    pub revenue: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportResponse {
    pub id: Uuid,
    pub report_type: String,
    pub format: String,
    pub data: serde_json::Value,
    pub generated_at: DateTime<Utc>,
}
