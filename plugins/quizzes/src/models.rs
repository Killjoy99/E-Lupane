use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Quiz {
    pub id: Uuid,
    pub course_id: Uuid,
    pub title: String,
    pub description: String,
    pub instructions: String,
    pub time_limit_minutes: Option<i32>,
    pub max_attempts: i32,
    pub passing_score: f64,
    pub randomize_questions: bool,
    pub randomize_answers: bool,
    pub negative_marking: bool,
    pub negative_mark_value: f64,
    pub show_results: String,
    pub allow_review: bool,
    pub status: String,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizQuestion {
    pub id: Uuid,
    pub quiz_id: Uuid,
    pub question_type: String,
    pub title: String,
    pub body: String,
    pub points: f64,
    pub sort_order: i32,
    pub correct_answer: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizQuestionOption {
    pub id: Uuid,
    pub question_id: Uuid,
    pub label: String,
    pub value: String,
    pub is_correct: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizAttempt {
    pub id: Uuid,
    pub quiz_id: Uuid,
    pub student_id: Uuid,
    pub attempt_number: i32,
    pub started_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub time_expired_at: Option<DateTime<Utc>>,
    pub score: Option<f64>,
    pub max_score: Option<f64>,
    pub passed: Option<bool>,
    pub graded: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AttemptAnswer {
    pub id: Uuid,
    pub attempt_id: Uuid,
    pub question_id: Uuid,
    pub answer: serde_json::Value,
    pub is_correct: Option<bool>,
    pub points_awarded: Option<f64>,
    pub graded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizAssignment {
    pub id: Uuid,
    pub quiz_id: Uuid,
    pub course_id: Uuid,
    pub available_from: Option<DateTime<Utc>>,
    pub available_until: Option<DateTime<Utc>>,
}
