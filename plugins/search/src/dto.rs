//! Data transfer objects for search plugin.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Search courses by title, description, or tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseSearchRequest {
    pub query: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Course search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseSearchResult {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub rank: f64,
}

/// Course search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseSearchResponse {
    pub results: Vec<CourseSearchResult>,
    pub total: i64,
}

/// Search content items by title or body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSearchRequest {
    pub query: String,
    pub course_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Content search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSearchResult {
    pub id: Uuid,
    pub title: String,
    pub body_preview: Option<String>,
    pub course_id: Uuid,
    pub rank: f64,
}

/// Content search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSearchResponse {
    pub results: Vec<ContentSearchResult>,
    pub total: i64,
}

/// Search all (courses + content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAllRequest {
    pub query: String,
    pub course_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Combined search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAllResponse {
    pub courses: Vec<CourseSearchResult>,
    pub content: Vec<ContentSearchResult>,
    pub total_courses: i64,
    pub total_content: i64,
}
