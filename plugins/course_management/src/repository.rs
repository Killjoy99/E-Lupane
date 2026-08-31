//! SQLx data access for courses and course instructors.

use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use sqlx::AssertSqlSafe;
use uuid::Uuid;

use crate::models::Course;

pub const COURSE_COLUMNS: &str = "id, code, title, short_description, description, thumbnail, \
    banner, category, subject, level, language, difficulty, duration, learning_objectives, \
    target_audience, prerequisites, tags, status, owner_id, created_by, updated_by, \
    created_at, updated_at";

pub async fn find_course_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Course>> {
    Ok(sqlx::query_as::<_, Course>(AssertSqlSafe(format!(
        "SELECT {COURSE_COLUMNS} FROM courses WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_course_by_code(pool: &PgPool, code: &str) -> Result<Option<Course>> {
    Ok(sqlx::query_as::<_, Course>(AssertSqlSafe(format!(
        "SELECT {COURSE_COLUMNS} FROM courses WHERE lower(code) = lower($1)"
    )))
    .bind(code)
    .fetch_optional(pool)
    .await?)
}

/// List courses, optionally filtered by status / owned-or-managed / search.
/// All filters are bound parameters (never interpolated).
pub async fn list_courses(
    pool: &PgPool,
    status: Option<&str>,
    mine: Option<Uuid>,
    search: Option<&str>,
    limit: i64,
) -> Result<Vec<Course>> {
    Ok(sqlx::query_as::<_, Course>(AssertSqlSafe(format!(
        "SELECT {COURSE_COLUMNS} FROM courses \
         WHERE ($1::text IS NULL OR status = $1) \
           AND ($2::uuid IS NULL OR owner_id = $2 OR EXISTS ( \
               SELECT 1 FROM course_instructors ci \
               WHERE ci.course_id = courses.id AND ci.user_id = $2)) \
           AND ($3::text IS NULL OR lower(code) LIKE lower('%' || $3 || '%') \
               OR lower(title) LIKE lower('%' || $3 || '%') \
               OR lower(subject) LIKE lower('%' || $3 || '%') \
               OR tags::text LIKE '%' || $3 || '%') \
         ORDER BY created_at DESC LIMIT $4"
    )))
    .bind(status)
    .bind(mine)
    .bind(search)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_course(
    pool: &PgPool,
    code: &str,
    title: &str,
    short_description: &str,
    description: &str,
    thumbnail: &str,
    banner: &str,
    category: &str,
    subject: &str,
    level: &str,
    language: &str,
    difficulty: &str,
    duration: &str,
    learning_objectives: &Value,
    target_audience: &Value,
    prerequisites: &Value,
    tags: &Value,
    owner_id: Uuid,
    created_by: Uuid,
) -> Result<Course> {
    Ok(sqlx::query_as::<_, Course>(AssertSqlSafe(format!(
        "INSERT INTO courses \
            (code, title, short_description, description, thumbnail, banner, category, subject, \
             level, language, difficulty, duration, learning_objectives, target_audience, \
             prerequisites, tags, owner_id, created_by, updated_by) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19) \
         RETURNING {COURSE_COLUMNS}"
    )))
    .bind(code)
    .bind(title)
    .bind(short_description)
    .bind(description)
    .bind(thumbnail)
    .bind(banner)
    .bind(category)
    .bind(subject)
    .bind(level)
    .bind(language)
    .bind(difficulty)
    .bind(duration)
    .bind(learning_objectives)
    .bind(target_audience)
    .bind(prerequisites)
    .bind(tags)
    .bind(owner_id)
    .bind(created_by)
    .bind(created_by)
    .fetch_one(pool)
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_course_fields(
    pool: &PgPool,
    id: Uuid,
    code: Option<&str>,
    title: Option<&str>,
    short_description: Option<&str>,
    description: Option<&str>,
    thumbnail: Option<&str>,
    banner: Option<&str>,
    category: Option<&str>,
    subject: Option<&str>,
    level: Option<&str>,
    language: Option<&str>,
    difficulty: Option<&str>,
    duration: Option<&str>,
    learning_objectives: Option<&Value>,
    target_audience: Option<&Value>,
    prerequisites: Option<&Value>,
    tags: Option<&Value>,
    updated_by: Uuid,
) -> Result<Course> {
    Ok(sqlx::query_as::<_, Course>(AssertSqlSafe(format!(
        "UPDATE courses SET \
            code = COALESCE($2, code), \
            title = COALESCE($3, title), \
            short_description = COALESCE($4, short_description), \
            description = COALESCE($5, description), \
            thumbnail = COALESCE($6, thumbnail), \
            banner = COALESCE($7, banner), \
            category = COALESCE($8, category), \
            subject = COALESCE($9, subject), \
            level = COALESCE($10, level), \
            language = COALESCE($11, language), \
            difficulty = COALESCE($12, difficulty), \
            duration = COALESCE($13, duration), \
            learning_objectives = COALESCE($14, learning_objectives), \
            target_audience = COALESCE($15, target_audience), \
            prerequisites = COALESCE($16, prerequisites), \
            tags = COALESCE($17, tags), \
            updated_by = $18, \
            updated_at = now() \
         WHERE id = $1 \
         RETURNING {COURSE_COLUMNS}"
    )))
    .bind(id)
    .bind(code)
    .bind(title)
    .bind(short_description)
    .bind(description)
    .bind(thumbnail)
    .bind(banner)
    .bind(category)
    .bind(subject)
    .bind(level)
    .bind(language)
    .bind(difficulty)
    .bind(duration)
    .bind(learning_objectives)
    .bind(target_audience)
    .bind(prerequisites)
    .bind(tags)
    .bind(updated_by)
    .fetch_one(pool)
    .await?)
}

pub async fn set_course_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    updated_by: Uuid,
) -> Result<Course> {
    Ok(sqlx::query_as::<_, Course>(AssertSqlSafe(format!(
        "UPDATE courses SET status = $2, updated_by = $3, updated_at = now() \
         WHERE id = $1 \
         RETURNING {COURSE_COLUMNS}"
    )))
    .bind(id)
    .bind(status)
    .bind(updated_by)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_course(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM courses WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Instructors
// ---------------------------------------------------------------------------

pub async fn add_instructor(
    pool: &PgPool,
    course_id: Uuid,
    user_id: Uuid,
    added_by: Uuid,
) -> Result<bool> {
    let res = sqlx::query(
        "INSERT INTO course_instructors (course_id, user_id, added_by) \
         VALUES ($1, $2, $3) ON CONFLICT (course_id, user_id) DO NOTHING",
    )
    .bind(course_id)
    .bind(user_id)
    .bind(added_by)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn remove_instructor(pool: &PgPool, course_id: Uuid, user_id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM course_instructors WHERE course_id = $1 AND user_id = $2")
        .bind(course_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn list_instructor_ids(pool: &PgPool, course_id: Uuid) -> Result<Vec<Uuid>> {
    let rows: Vec<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM course_instructors WHERE course_id = $1")
            .bind(course_id)
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}


