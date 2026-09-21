//! Domain models for search plugin.

/// Represents a search index entry for courses.
#[derive(Debug, Clone)]
pub struct CourseSearchEntry {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub search_vector: Option<String>,
}

/// Represents a search index entry for content items.
#[derive(Debug, Clone)]
pub struct ContentSearchEntry {
    pub id: uuid::Uuid,
    pub title: String,
    pub body: Option<String>,
    pub course_id: uuid::Uuid,
    pub search_vector: Option<String>,
}
