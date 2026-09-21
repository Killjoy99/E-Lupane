//! Database repository for search plugin.

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{ContentSearchResult, CourseSearchResult};

/// Search courses using Postgres full-text search.
pub async fn search_courses(
    pool: &PgPool,
    query: &str,
    limit: i64,
    offset: i64,
) -> anyhow::Result<(Vec<CourseSearchResult>, i64)> {
    let webquery = query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" | ");

    let rows = sqlx::query_as!(
        CourseSearchRow,
        r#"
        SELECT id,
               title,
               description,
               ts_rank(search_vector, websearch_to_tsquery('english', $1)) AS rank
        FROM courses
        WHERE search_vector @@ websearch_to_tsquery('english', $1)
        ORDER BY rank DESC
        LIMIT $2 OFFSET $3
        "#,
        webquery,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM courses
        WHERE search_vector @@ websearch_to_tsquery('english', $1)
        "#,
        webquery,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let results = rows.into_iter().map(|r| r.into()).collect();
    Ok((results, total))
}

/// Search content items using Postgres full-text search.
pub async fn search_content(
    pool: &PgPool,
    query: &str,
    course_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> anyhow::Result<(Vec<ContentSearchResult>, i64)> {
    let webquery = query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" | ");

    let rows = sqlx::query_as!(
        ContentSearchRow,
        r#"
        SELECT ci.id,
               ci.title,
               LEFT(ci.body, 200) AS body_preview,
               cm.course_id,
               ts_rank(ci.search_vector, websearch_to_tsquery('english', $1)) AS rank
        FROM content_items ci
        JOIN content_lessons cl ON cl.id = ci.lesson_id
        JOIN content_modules cm ON cm.id = cl.module_id
        WHERE ci.search_vector @@ websearch_to_tsquery('english', $1)
          AND ($2::uuid IS NULL OR cm.course_id = $2)
        ORDER BY rank DESC
        LIMIT $3 OFFSET $4
        "#,
        webquery,
        course_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM content_items ci
        JOIN content_lessons cl ON cl.id = ci.lesson_id
        JOIN content_modules cm ON cm.id = cl.module_id
        WHERE ci.search_vector @@ websearch_to_tsquery('english', $1)
          AND ($2::uuid IS NULL OR cm.course_id = $2)
        "#,
        webquery,
        course_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let results = rows.into_iter().map(|r| r.into()).collect();
    Ok((results, total))
}

// Internal row types

#[derive(sqlx::FromRow)]
struct CourseSearchRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    rank: Option<f32>,
}

impl From<CourseSearchRow> for CourseSearchResult {
    fn from(row: CourseSearchRow) -> Self {
        CourseSearchResult {
            id: row.id,
            title: row.title,
            description: row.description,
            rank: row.rank.map(|r| r as f64).unwrap_or(0.0),
        }
    }
}

#[derive(sqlx::FromRow)]
struct ContentSearchRow {
    id: Uuid,
    title: String,
    body_preview: Option<String>,
    course_id: Uuid,
    rank: Option<f32>,
}

impl From<ContentSearchRow> for ContentSearchResult {
    fn from(row: ContentSearchRow) -> Self {
        ContentSearchResult {
            id: row.id,
            title: row.title,
            body_preview: row.body_preview,
            course_id: row.course_id,
            rank: row.rank.map(|r| r as f64).unwrap_or(0.0),
        }
    }
}
