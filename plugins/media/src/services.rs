//! Business logic for media plugin.
//!
//! Uses local filesystem storage on the server instead of cloud storage.

use std::path::PathBuf;
use std::sync::OnceLock;

use sha2::Digest;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{
    CompleteUploadRequest, CompleteUploadResponse, DownloadResponse, FileInfo, FileListResponse,
    UploadResponse,
};
use crate::models::FileStatus;
use crate::repository;

static STORAGE_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Initialize the storage path from environment variables.
pub fn init_storage_path() -> anyhow::Result<()> {
    let path = std::env::var("MEDIA_STORAGE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./media-storage"));

    std::fs::create_dir_all(&path)?;
    STORAGE_PATH.set(path).map_err(|_| anyhow::anyhow!("Storage path already initialized"))?;
    Ok(())
}

/// Get the storage root path.
fn storage_path() -> &'static PathBuf {
    STORAGE_PATH.get().expect("Storage path not initialized")
}

/// Get the full path for a file.
fn file_path(file_id: &Uuid, filename: &str) -> PathBuf {
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    storage_path().join(format!("{}.{}", file_id, extension))
}

/// Request a file upload - returns upload URL and token.
pub async fn request_upload(
    pool: &PgPool,
    user_id: Uuid,
    req: &crate::dto::UploadRequest,
) -> anyhow::Result<UploadResponse> {
    let file_id = Uuid::new_v4();

    repository::insert_file(
        pool,
        &req.filename,
        &req.mime_type,
        req.size_bytes,
        req.course_id,
        user_id,
        &file_id.to_string(),
    )
    .await?;

    let upload_token = generate_upload_token(&file_id, user_id);

    Ok(UploadResponse {
        file_id,
        upload_url: format!("/api/media/upload/{}", file_id),
        upload_token: Some(upload_token),
        expires_in_seconds: 900,
    })
}

/// Confirm upload completion and update status.
pub async fn complete_upload(
    pool: &PgPool,
    user_id: Uuid,
    req: &CompleteUploadRequest,
) -> anyhow::Result<CompleteUploadResponse> {
    let file = repository::get_file(pool, req.file_id).await?;
    let file = file.ok_or_else(|| anyhow::anyhow!("file not found"))?;

    if file.uploaded_by != user_id {
        return Err(anyhow::anyhow!("access denied"));
    }

    let path = file_path(&req.file_id, &file.filename);
    if !path.exists() {
        return Err(anyhow::anyhow!("file not found on storage"));
    }

    let metadata = std::fs::metadata(&path)?;
    let size_bytes = metadata.len() as i64;

    let content = std::fs::read(&path)?;
    let hash = sha2::Sha256::digest(&content);
    let etag = format!("{:x}", hash);

    let updated = repository::mark_complete(pool, req.file_id, &etag, size_bytes).await?;
    let file = updated.ok_or_else(|| anyhow::anyhow!("file not found"))?;

    Ok(CompleteUploadResponse {
        file_id: file.id,
        status: "complete".to_string(),
        size_bytes,
    })
}

/// Generate a download URL.
pub async fn request_download(
    pool: &PgPool,
    user_id: Uuid,
    req: &crate::dto::DownloadRequest,
) -> anyhow::Result<DownloadResponse> {
    let file = repository::get_file(pool, req.file_id).await?;
    let file = file.ok_or_else(|| anyhow::anyhow!("file not found"))?;

    if file.uploaded_by != user_id {
        return Err(anyhow::anyhow!("access denied"));
    }

    if file.status != FileStatus::Complete {
        return Err(anyhow::anyhow!("file not ready for download"));
    }

    let download_token = generate_download_token(&req.file_id, user_id);

    Ok(DownloadResponse {
        file_id: file.id,
        download_url: format!("/api/media/download/{}", file.id),
        download_token: Some(download_token),
        filename: file.filename,
        mime_type: file.mime_type,
        expires_in_seconds: 3600,
    })
}

/// Delete a file (soft delete in DB, remove from disk).
pub async fn delete_file(
    pool: &PgPool,
    user_id: Uuid,
    is_admin: bool,
    file_id: &Uuid,
) -> anyhow::Result<()> {
    let file = repository::get_file(pool, *file_id).await?;
    let file = file.ok_or_else(|| anyhow::anyhow!("file not found"))?;

    if file.uploaded_by != user_id && !is_admin {
        return Err(anyhow::anyhow!("access denied"));
    }

    let path = file_path(file_id, &file.filename);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }

    repository::soft_delete(pool, *file_id).await?;
    Ok(())
}

/// List files with pagination.
pub async fn list_files(
    pool: &PgPool,
    user_id: Uuid,
    is_admin: bool,
    req: &crate::dto::ListFilesRequest,
) -> anyhow::Result<FileListResponse> {
    let limit = req.limit.unwrap_or(50).min(100);
    let offset = req.offset.unwrap_or(0);

    let (files, total) = repository::list_files(pool, req.course_id, limit, offset).await?;

    let files = if is_admin {
        files
    } else {
        files.into_iter().filter(|f| f.uploaded_by == user_id).collect()
    };

    Ok(FileListResponse { files, total })
}

/// Get file info.
pub async fn get_file_info(
    pool: &PgPool,
    user_id: Uuid,
    is_admin: bool,
    file_id: &Uuid,
) -> anyhow::Result<FileInfo> {
    let file = repository::get_file(pool, *file_id).await?;
    let file = file.ok_or_else(|| anyhow::anyhow!("file not found"))?;

    if file.uploaded_by != user_id && !is_admin {
        return Err(anyhow::anyhow!("access denied"));
    }

    Ok(FileInfo {
        id: file.id,
        filename: file.filename,
        mime_type: file.mime_type,
        size_bytes: file.size_bytes,
        course_id: file.course_id,
        uploaded_by: file.uploaded_by,
        created_at: file.created_at,
        etag: file.etag,
    })
}

/// Write file content to disk (called by upload handler).
pub async fn write_file(file_id: &Uuid, filename: &str, content: &[u8]) -> anyhow::Result<()> {
    let path = file_path(file_id, filename);
    tokio::fs::write(&path, content).await?;
    Ok(())
}

/// Read file content from disk (called by download handler).
pub async fn read_file(file_id: &Uuid, filename: &str) -> anyhow::Result<Vec<u8>> {
    let path = file_path(file_id, filename);
    let content = tokio::fs::read(&path).await?;
    Ok(content)
}

/// Generate a simple upload token.
fn generate_upload_token(file_id: &Uuid, user_id: Uuid) -> String {
    let data = format!("upload:{}:{}:{}", file_id, user_id, chrono::Utc::now().timestamp());
    let hash = sha2::Sha256::digest(data.as_bytes());
    format!("{:x}", hash)
}

/// Generate a simple download token.
fn generate_download_token(file_id: &Uuid, user_id: Uuid) -> String {
    let data = format!("download:{}:{}:{}", file_id, user_id, chrono::Utc::now().timestamp());
    let hash = sha2::Sha256::digest(data.as_bytes());
    format!("{:x}", hash)
}


