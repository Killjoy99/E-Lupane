use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuizRequest {
    pub course_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub time_limit_minutes: Option<i32>,
    pub max_attempts: i32,
    pub passing_score: f64,
    pub randomize_questions: bool,
    pub randomize_answers: bool,
    pub negative_marking: bool,
    pub negative_mark_value: f64,
    pub show_results: String,
    pub allow_review: bool,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQuizRequest {
    pub id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub time_limit_minutes: Option<i32>,
    pub max_attempts: Option<i32>,
    pub passing_score: Option<f64>,
    pub randomize_questions: Option<bool>,
    pub randomize_answers: Option<bool>,
    pub negative_marking: Option<bool>,
    pub negative_mark_value: Option<f64>,
    pub show_results: Option<String>,
    pub allow_review: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuestionRequest {
    pub quiz_id: Uuid,
    pub question_type: String,
    pub title: String,
    pub body: Option<String>,
    pub points: f64,
    pub sort_order: Option<i32>,
    pub correct_answer: serde_json::Value,
    pub options: Vec<CreateQuestionOptionRequest>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuestionOptionRequest {
    pub label: Option<String>,
    pub value: String,
    pub is_correct: bool,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartAttemptRequest {
    pub quiz_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitAnswerRequest {
    pub attempt_id: Uuid,
    pub question_id: Uuid,
    pub answer: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitAttemptRequest {
    pub attempt_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizResponse {
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestionResponse {
    pub id: Uuid,
    pub quiz_id: Uuid,
    pub question_type: String,
    pub title: String,
    pub body: String,
    pub points: f64,
    pub sort_order: i32,
    pub options: Vec<QuizQuestionOptionResponse>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestionOptionResponse {
    pub id: Uuid,
    pub question_id: Uuid,
    pub label: String,
    pub value: String,
    pub is_correct: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizAttemptResponse {
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
    pub answers: Vec<AttemptAnswerResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptAnswerResponse {
    pub id: Uuid,
    pub attempt_id: Uuid,
    pub question_id: Uuid,
    pub answer: serde_json::Value,
    pub is_correct: Option<bool>,
    pub points_awarded: Option<f64>,
    pub graded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizStudentView {
    pub quiz: QuizResponse,
    pub attempt: Option<QuizAttemptResponse>,
    pub questions: Vec<QuizQuestionResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseIdRequest {
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdRequest {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizIdRequest {
    pub quiz_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptIdRequest {
    pub attempt_id: Uuid,
}
