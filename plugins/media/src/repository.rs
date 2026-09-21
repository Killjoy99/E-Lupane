//! Database repository for media_service plugin.

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::FileInfo;
use crate::models::{FileStatus, MediaFile};

/// Insert a new pending file record.
pub async fn insert_file(
    pool: &PgPool,
    filename: &str,
    mime_type: &str,
    size_bytes: i64,
    course_id: Option<Uuid>,
    uploaded_by: Uuid,
    storage_key: &str,
) -> anyhow::Result<MediaFile> {
    let row = sqlx::query_as!(
        MediaFileRow,
        r#"
        INSERT INTO media_files (filename, mime_type, size_bytes, course_id, uploaded_by, storage_key, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'pending')
        RETURNING id, filename, mime_type, size_bytes, course_id, uploaded_by, storage_key, etag, status, created_at, updated_at
        "#,
        filename,
        mime_type,
        size_bytes,
        course_id,
        uploaded_by,
        storage_key,
    )
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

/// Mark file as complete with etag and actual size.
pub async fn mark_complete(pool: &PgPool, file_id: Uuid, etag: &str, size_bytes: i64) -> anyhow::Result<Option<MediaFile>> {
    let row = sqlx::query_as!(
        MediaFileRow,
        r#"
        UPDATE media_files
        SET status = 'complete', etag = $2, size_bytes = $3, updated_at = NOW()
        WHERE id = $1
        RETURNING id, filename, mime_type, size_bytes, course_id, uploaded_by, storage_key, etag, status, created_at, updated_at
        "#,
        file_id,
        etag,
        size_bytes,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.into()))
}

/// Soft-delete a file record.
pub async fn soft_delete(pool: &PgPool, file_id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE media_files
        SET status = 'deleted', updated_at = NOW()
        WHERE id = $1 AND status != 'deleted'
        "#,
        file_id,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Get file by ID.
pub async fn get_file(pool: &PgPool, file_id: Uuid) -> anyhow::Result<Option<MediaFile>> {
    let row = sqlx::query_as!(
        MediaFileRow,
        r#"
        SELECT id, filename, mime_type, size_bytes, course_id, uploaded_by, storage_key, etag, status, created_at, updated_at
        FROM media_files
        WHERE id = $1 AND status != 'deleted'
        "#,
        file_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.into()))
}

/// List files with pagination.
pub async fn list_files(
    pool: &PgPool,
    course_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> anyhow::Result<(Vec<FileInfo>, i64)> {
    let rows = sqlx::query_as!(
        FileInfoRow,
        r#"
        SELECT id, filename, mime_type, size_bytes, course_id, uploaded_by, created_at
        FROM media_files
        WHERE status != 'deleted' AND ($1::uuid IS NULL OR course_id = $1)
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        course_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM media_files
        WHERE status != 'deleted' AND ($1::uuid IS NULL OR course_id = $1)
        "#,
        course_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let files = rows.into_iter().map(|r| r.into()).collect();
    Ok((files, total))
}

// Internal row types for SQLx mapping

#[derive(sqlx::FromRow)]
struct MediaFileRow {
    id: Uuid,
    filename: String,
    mime_type: String,
    size_bytes: i64,
    course_id: Option<Uuid>,
    uploaded_by: Uuid,
    storage_key: String,
    etag: Option<String>,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<MediaFileRow> for MediaFile {
    fn from(row: MediaFileRow) -> Self {
        MediaFile {
            id: row.id,
            filename: row.filename,
            mime_type: row.mime_type,
            size_bytes: row.size_bytes,
            course_id: row.course_id,
            uploaded_by: row.uploaded_by,
            storage_key: row.storage_key,
            etag: row.etag,
            status: row.status.parse().unwrap_or(FileStatus::Pending),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct FileInfoRow {
    id: Uuid,
    filename: String,
    mime_type: String,
    size_bytes: i64,
    course_id: Option<Uuid>,
    uploaded_by: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<FileInfoRow> for FileInfo {
    fn from(row: FileInfoRow) -> Self {
        FileInfo {
            id: row.id,
            filename: row.filename,
            mime_type: row.mime_type,
            size_bytes: row.size_bytes,
            course_id: row.course_id,
            uploaded_by: row.uploaded_by,
            created_at: row.created_at,
            etag: None,
        }
    }
}
