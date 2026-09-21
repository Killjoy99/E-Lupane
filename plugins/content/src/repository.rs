//! SQLx data access for content modules, lessons, and content items.

use anyhow::Result;
use sqlx::PgPool;
use sqlx::AssertSqlSafe;
use uuid::Uuid;

use crate::models::{ContentFile, ContentItem, ContentLesson, ContentModule};

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

pub const MODULE_COLUMNS: &str =
    "id, course_id, title, description, sort_order, status, published_at, created_by, updated_by, created_at, updated_at";

pub async fn find_module_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ContentModule>> {
    Ok(sqlx::query_as::<_, ContentModule>(AssertSqlSafe(format!(
        "SELECT {MODULE_COLUMNS} FROM content_modules WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_modules_by_course(pool: &PgPool, course_id: Uuid) -> Result<Vec<ContentModule>> {
    Ok(sqlx::query_as::<_, ContentModule>(AssertSqlSafe(format!(
        "SELECT {MODULE_COLUMNS} FROM content_modules WHERE course_id = $1 ORDER BY sort_order ASC, created_at ASC"
    )))
    .bind(course_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_module(
    pool: &PgPool,
    course_id: Uuid,
    title: &str,
    description: &str,
    sort_order: i32,
    created_by: Uuid,
    updated_by: Uuid,
) -> Result<ContentModule> {
    Ok(sqlx::query_as::<_, ContentModule>(AssertSqlSafe(format!(
        "INSERT INTO content_modules (course_id, title, description, sort_order, created_by, updated_by) \
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING {MODULE_COLUMNS}"
    )))
    .bind(course_id)
    .bind(title)
    .bind(description)
    .bind(sort_order)
    .bind(created_by)
    .bind(updated_by)
    .fetch_one(pool)
    .await?)
}

pub async fn update_module_fields(
    pool: &PgPool,
    id: Uuid,
    title: Option<&str>,
    description: Option<&str>,
    sort_order: Option<i32>,
    updated_by: Uuid,
) -> Result<ContentModule> {
    Ok(sqlx::query_as::<_, ContentModule>(AssertSqlSafe(format!(
        "UPDATE content_modules SET \
            title = COALESCE($2, title), \
            description = COALESCE($3, description), \
            sort_order = COALESCE($4, sort_order), \
            updated_by = $5, \
            updated_at = now() \
         WHERE id = $1 \
         RETURNING {MODULE_COLUMNS}"
    )))
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(sort_order)
    .bind(updated_by)
    .fetch_one(pool)
    .await?)
}

pub async fn set_module_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    updated_by: Uuid,
) -> Result<ContentModule> {
    let published_at = if status == "published" {
        Some("now()")
    } else {
        None
    };

    if let Some(pa) = published_at {
        Ok(sqlx::query_as::<_, ContentModule>(AssertSqlSafe(format!(
            "UPDATE content_modules SET status = $2, published_at = {pa}, updated_by = $3, updated_at = now() \
             WHERE id = $1 RETURNING {MODULE_COLUMNS}"
        )))
        .bind(id)
        .bind(status)
        .bind(updated_by)
        .fetch_one(pool)
        .await?)
    } else {
        Ok(sqlx::query_as::<_, ContentModule>(AssertSqlSafe(format!(
            "UPDATE content_modules SET status = $2, updated_by = $3, updated_at = now() \
             WHERE id = $1 RETURNING {MODULE_COLUMNS}"
        )))
        .bind(id)
        .bind(status)
        .bind(updated_by)
        .fetch_one(pool)
        .await?)
    }
}

pub async fn delete_module(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM content_modules WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Lessons
// ---------------------------------------------------------------------------

pub const LESSON_COLUMNS: &str =
    "id, module_id, title, description, sort_order, status, published_at, created_by, updated_by, created_at, updated_at";

pub async fn find_lesson_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ContentLesson>> {
    Ok(sqlx::query_as::<_, ContentLesson>(AssertSqlSafe(format!(
        "SELECT {LESSON_COLUMNS} FROM content_lessons WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_lessons_by_module(pool: &PgPool, module_id: Uuid) -> Result<Vec<ContentLesson>> {
    Ok(sqlx::query_as::<_, ContentLesson>(AssertSqlSafe(format!(
        "SELECT {LESSON_COLUMNS} FROM content_lessons WHERE module_id = $1 ORDER BY sort_order ASC, created_at ASC"
    )))
    .bind(module_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_lesson(
    pool: &PgPool,
    module_id: Uuid,
    title: &str,
    description: &str,
    sort_order: i32,
    created_by: Uuid,
    updated_by: Uuid,
) -> Result<ContentLesson> {
    Ok(sqlx::query_as::<_, ContentLesson>(AssertSqlSafe(format!(
        "INSERT INTO content_lessons (module_id, title, description, sort_order, created_by, updated_by) \
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING {LESSON_COLUMNS}"
    )))
    .bind(module_id)
    .bind(title)
    .bind(description)
    .bind(sort_order)
    .bind(created_by)
    .bind(updated_by)
    .fetch_one(pool)
    .await?)
}

pub async fn update_lesson_fields(
    pool: &PgPool,
    id: Uuid,
    title: Option<&str>,
    description: Option<&str>,
    sort_order: Option<i32>,
    updated_by: Uuid,
) -> Result<ContentLesson> {
    Ok(sqlx::query_as::<_, ContentLesson>(AssertSqlSafe(format!(
        "UPDATE content_lessons SET \
            title = COALESCE($2, title), \
            description = COALESCE($3, description), \
            sort_order = COALESCE($4, sort_order), \
            updated_by = $5, \
            updated_at = now() \
         WHERE id = $1 \
         RETURNING {LESSON_COLUMNS}"
    )))
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(sort_order)
    .bind(updated_by)
    .fetch_one(pool)
    .await?)
}

pub async fn set_lesson_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    updated_by: Uuid,
) -> Result<ContentLesson> {
    let published_at = if status == "published" {
        Some("now()")
    } else {
        None
    };

    if let Some(pa) = published_at {
        Ok(sqlx::query_as::<_, ContentLesson>(AssertSqlSafe(format!(
            "UPDATE content_lessons SET status = $2, published_at = {pa}, updated_by = $3, updated_at = now() \
             WHERE id = $1 RETURNING {LESSON_COLUMNS}"
        )))
        .bind(id)
        .bind(status)
        .bind(updated_by)
        .fetch_one(pool)
        .await?)
    } else {
        Ok(sqlx::query_as::<_, ContentLesson>(AssertSqlSafe(format!(
            "UPDATE content_lessons SET status = $2, updated_by = $3, updated_at = now() \
             WHERE id = $1 RETURNING {LESSON_COLUMNS}"
        )))
        .bind(id)
        .bind(status)
        .bind(updated_by)
        .fetch_one(pool)
        .await?)
    }
}

pub async fn delete_lesson(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM content_lessons WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Content Items
// ---------------------------------------------------------------------------

pub const ITEM_COLUMNS: &str =
    "id, lesson_id, content_type, title, body, url, file_id, mime_type, file_size_bytes, sort_order, created_by, updated_by, created_at, updated_at";

pub async fn find_item_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ContentItem>> {
    Ok(sqlx::query_as::<_, ContentItem>(AssertSqlSafe(format!(
        "SELECT {ITEM_COLUMNS} FROM content_items WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_items_by_lesson(pool: &PgPool, lesson_id: Uuid) -> Result<Vec<ContentItem>> {
    Ok(sqlx::query_as::<_, ContentItem>(AssertSqlSafe(format!(
        "SELECT {ITEM_COLUMNS} FROM content_items WHERE lesson_id = $1 ORDER BY sort_order ASC, created_at ASC"
    )))
    .bind(lesson_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_item(
    pool: &PgPool,
    lesson_id: Uuid,
    content_type: &str,
    title: &str,
    body: &str,
    url: &str,
    file_id: Option<Uuid>,
    mime_type: &str,
    sort_order: i32,
    created_by: Uuid,
    updated_by: Uuid,
) -> Result<ContentItem> {
    Ok(sqlx::query_as::<_, ContentItem>(AssertSqlSafe(format!(
        "INSERT INTO content_items (lesson_id, content_type, title, body, url, file_id, mime_type, sort_order, created_by, updated_by) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING {ITEM_COLUMNS}"
    )))
    .bind(lesson_id)
    .bind(content_type)
    .bind(title)
    .bind(body)
    .bind(url)
    .bind(file_id)
    .bind(mime_type)
    .bind(sort_order)
    .bind(created_by)
    .bind(updated_by)
    .fetch_one(pool)
    .await?)
}

pub async fn update_item_fields(
    pool: &PgPool,
    id: Uuid,
    content_type: Option<&str>,
    title: Option<&str>,
    body: Option<&str>,
    url: Option<&str>,
    file_id: Option<Uuid>,
    mime_type: Option<&str>,
    sort_order: Option<i32>,
    updated_by: Uuid,
) -> Result<ContentItem> {
    Ok(sqlx::query_as::<_, ContentItem>(AssertSqlSafe(format!(
        "UPDATE content_items SET \
            content_type = COALESCE($2, content_type), \
            title = COALESCE($3, title), \
            body = COALESCE($4, body), \
            url = COALESCE($5, url), \
            file_id = COALESCE($6, file_id), \
            mime_type = COALESCE($7, mime_type), \
            sort_order = COALESCE($8, sort_order), \
            updated_by = $9, \
            updated_at = now() \
         WHERE id = $1 \
         RETURNING {ITEM_COLUMNS}"
    )))
    .bind(id)
    .bind(content_type)
    .bind(title)
    .bind(body)
    .bind(url)
    .bind(file_id)
    .bind(mime_type)
    .bind(sort_order)
    .bind(updated_by)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_item(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM content_items WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Content Files
// ---------------------------------------------------------------------------

pub const FILE_COLUMNS: &str =
    "id, course_id, filename, mime_type, size_bytes, storage_path, created_by, created_at";

pub async fn find_file_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ContentFile>> {
    Ok(sqlx::query_as::<_, ContentFile>(AssertSqlSafe(format!(
        "SELECT {FILE_COLUMNS} FROM content_files WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_files_by_course(pool: &PgPool, course_id: Uuid) -> Result<Vec<ContentFile>> {
    Ok(sqlx::query_as::<_, ContentFile>(AssertSqlSafe(format!(
        "SELECT {FILE_COLUMNS} FROM content_files WHERE course_id = $1 ORDER BY created_at DESC"
    )))
    .bind(course_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_file(
    pool: &PgPool,
    course_id: Uuid,
    filename: &str,
    mime_type: &str,
    size_bytes: i64,
    storage_path: &str,
    created_by: Uuid,
) -> Result<ContentFile> {
    Ok(sqlx::query_as::<_, ContentFile>(AssertSqlSafe(format!(
        "INSERT INTO content_files (course_id, filename, mime_type, size_bytes, storage_path, created_by) \
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING {FILE_COLUMNS}"
    )))
    .bind(course_id)
    .bind(filename)
    .bind(mime_type)
    .bind(size_bytes)
    .bind(storage_path)
    .bind(created_by)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_file(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM content_files WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Course access control (shared read model across content objects)
// ---------------------------------------------------------------------------

/// The caller's level of access to a course's content.
///
/// * `None`   — not the owner/instructor and not an active enrollee.
/// * `View`   — actively enrolled student: may read/download content.
/// * `Manage` — course owner or an authorised instructor: may author content.
///
/// Admins are treated as `Manage` by the service layer (they are not stored on
/// the `courses` row, so they are checked from the JWT roles before this query).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CourseAccess {
    None,
    View,
    Manage,
}

/// Resolve how much access `user_id` has to `course_id`'s content.
pub async fn course_access(pool: &PgPool, user_id: Uuid, course_id: Uuid) -> Result<CourseAccess> {
    let lvl: Option<String> = sqlx::query_scalar(
        "SELECT CASE \
            WHEN EXISTS (SELECT 1 FROM courses c \
                         WHERE c.id = $2 AND (c.owner_id = $1 OR c.created_by = $1)) THEN 'manage' \
            WHEN EXISTS (SELECT 1 FROM course_instructors ci \
                         WHERE ci.course_id = $2 AND ci.user_id = $1) THEN 'manage' \
            WHEN EXISTS (SELECT 1 FROM course_enrollments ce \
                         WHERE ce.course_id = $2 AND ce.student_id = $1 AND ce.state = 'active') THEN 'view' \
            ELSE 'none' \
         END",
    )
    .bind(user_id)
    .bind(course_id)
    .fetch_one(pool)
    .await?;

    Ok(match lvl.as_deref() {
        Some("manage") => CourseAccess::Manage,
        Some("view") => CourseAccess::View,
        _ => CourseAccess::None,
    })
}

/// The parent course of a module (if any).
pub async fn module_course_id(pool: &PgPool, module_id: Uuid) -> Result<Option<Uuid>> {
    Ok(sqlx::query_scalar("SELECT course_id FROM content_modules WHERE id = $1")
        .bind(module_id)
        .fetch_optional(pool)
        .await?)
}

/// The parent course of a lesson (via its module).
pub async fn lesson_course_id(pool: &PgPool, lesson_id: Uuid) -> Result<Option<Uuid>> {
    Ok(sqlx::query_scalar(
        "SELECT m.course_id \
         FROM content_lessons l \
         JOIN content_modules m ON m.id = l.module_id \
         WHERE l.id = $1",
    )
    .bind(lesson_id)
    .fetch_optional(pool)
    .await?)
}

/// The parent course of a content item (via lesson -> module).
pub async fn item_course_id(pool: &PgPool, item_id: Uuid) -> Result<Option<Uuid>> {
    Ok(sqlx::query_scalar(
        "SELECT m.course_id \
         FROM content_items i \
         JOIN content_lessons l ON l.id = i.lesson_id \
         JOIN content_modules m ON m.id = l.module_id \
         WHERE i.id = $1",
    )
    .bind(item_id)
    .fetch_optional(pool)
    .await?)
}

/// Find a module by its course and title (used to idempotently ensure a
/// well-known "Course Materials" container when a creator uploads content).
pub async fn find_module_by_course_title(
    pool: &PgPool,
    course_id: Uuid,
    title: &str,
) -> Result<Option<ContentModule>> {
    Ok(sqlx::query_as::<_, ContentModule>(AssertSqlSafe(format!(
        "SELECT {MODULE_COLUMNS} FROM content_modules WHERE course_id = $1 AND title = $2 ORDER BY created_at LIMIT 1"
    )))
    .bind(course_id)
    .bind(title)
    .fetch_optional(pool)
    .await?)
}

