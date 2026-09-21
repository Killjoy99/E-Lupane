//! Business logic for search plugin.

use sqlx::PgPool;

use crate::dto::{
    ContentSearchRequest, ContentSearchResponse, CourseSearchRequest, CourseSearchResponse,
    SearchAllRequest, SearchAllResponse,
};
use crate::repository;

/// Search courses by query.
pub async fn search_courses(
    pool: &PgPool,
    req: &CourseSearchRequest,
) -> anyhow::Result<CourseSearchResponse> {
    let limit = req.limit.unwrap_or(20).min(100);
    let offset = req.offset.unwrap_or(0);

    let (results, total) = repository::search_courses(pool, &req.query, limit, offset).await?;

    Ok(CourseSearchResponse { results, total })
}

/// Search content items by query.
pub async fn search_content(
    pool: &PgPool,
    req: &ContentSearchRequest,
) -> anyhow::Result<ContentSearchResponse> {
    let limit = req.limit.unwrap_or(20).min(100);
    let offset = req.offset.unwrap_or(0);

    let (results, total) = repository::search_content(pool, &req.query, req.course_id, limit, offset).await?;

    Ok(ContentSearchResponse { results, total })
}

/// Search both courses and content items.
pub async fn search_all(
    pool: &PgPool,
    req: &SearchAllRequest,
) -> anyhow::Result<SearchAllResponse> {
    let limit = req.limit.unwrap_or(10).min(50);
    let offset = req.offset.unwrap_or(0);

    let (courses, total_courses) = repository::search_courses(pool, &req.query, limit, offset).await?;
    let (content, total_content) = repository::search_content(pool, &req.query, req.course_id, limit, offset).await?;

    Ok(SearchAllResponse {
        courses,
        content,
        total_courses,
        total_content,
    })
}
