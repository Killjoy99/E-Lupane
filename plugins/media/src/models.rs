//! Domain models for media_service plugin.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents a stored file record.
#[derive(Debug, Clone)]
pub struct MediaFile {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub course_id: Option<Uuid>,
    pub uploaded_by: Uuid,
    pub storage_key: String,
    pub etag: Option<String>,
    pub status: FileStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// File upload status.
#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    /// Upload URL generated, awaiting client upload.
    Pending,
    /// Upload completed, file available.
    Complete,
    /// File deleted.
    Deleted,
}

impl FileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileStatus::Pending => "pending",
            FileStatus::Complete => "complete",
            FileStatus::Deleted => "deleted",
        }
    }
}

impl std::str::FromStr for FileStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(FileStatus::Pending),
            "complete" => Ok(FileStatus::Complete),
            "deleted" => Ok(FileStatus::Deleted),
            _ => Err(anyhow::anyhow!("unknown file status: {}", s)),
        }
    }
}
