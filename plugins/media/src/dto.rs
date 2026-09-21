//! Data transfer objects for media_service plugin.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request a presigned upload URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadRequest {
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub course_id: Option<Uuid>,
    pub content_type: Option<String>,
}

/// Response with upload URL and token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub file_id: Uuid,
    pub upload_url: String,
    pub upload_token: Option<String>,
    pub expires_in_seconds: i64,
}

/// Confirm upload completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteUploadRequest {
    pub file_id: Uuid,
}

/// Response after upload completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteUploadResponse {
    pub file_id: Uuid,
    pub status: String,
    pub size_bytes: i64,
}

/// Request a presigned download URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub file_id: Uuid,
}

/// Response with download URL and token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResponse {
    pub file_id: Uuid,
    pub download_url: String,
    pub download_token: Option<String>,
    pub filename: String,
    pub mime_type: String,
    pub expires_in_seconds: i64,
}

/// Delete a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFileRequest {
    pub file_id: Uuid,
}

/// List files with optional filters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesRequest {
    pub course_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// File list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListResponse {
    pub files: Vec<FileInfo>,
    pub total: i64,
}

/// Get file info request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfoRequest {
    pub file_id: Uuid,
}

/// File metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub course_id: Option<Uuid>,
    pub uploaded_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub etag: Option<String>,
}
