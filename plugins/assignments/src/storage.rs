//! Secure storage for uploaded files (assignment resources and submission
//! files).
//!
//! Files are stored under `<storage-dir>/<assignment_id>/<random-uuid>`. The
//! on-disk name is always server-generated; the original user-supplied name is
//! kept only in the database, so no user input ever reaches the filesystem
//! path. Traversal attempts are rejected up front and re-checked on read.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const DEFAULT_MAX_FILE_BYTES: usize = 25 * 1024 * 1024; // 25 MiB

/// Configurable per-file size limit (`ASSIGNMENTS_MAX_FILE_BYTES`).
pub fn max_file_bytes() -> usize {
    std::env::var("ASSIGNMENTS_MAX_FILE_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_FILE_BYTES)
}

/// Base directory for stored assignment files
/// (`ASSIGNMENTS_STORAGE_DIR`, default `data/assignments` under the CWD).
pub fn storage_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("ASSIGNMENTS_STORAGE_DIR")
            .unwrap_or_else(|_| "data/assignments".to_string()),
    )
}

/// Reject unsafe file names (path traversal, separators, control bytes).
pub fn sanitize_filename(name: &str) -> Result<String> {
    if name.is_empty() {
        return Err(anyhow!("file name must not be empty"));
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") || name.contains('\0') {
        return Err(anyhow!("unsafe file name"));
    }
    Ok(name.to_string())
}

/// The directory that holds one assignment's files (created if needed).
fn assignment_dir(assignment_id: Uuid) -> Result<PathBuf> {
    let dir = storage_dir().join(assignment_id.to_string());
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// sha256 digest of `bytes`, hex-encoded (used for integrity checks).
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Metadata describing a stored file.
#[derive(Debug, Clone)]
pub struct StoredFile {
    pub stored_name: String,
    pub content_hash: String,
    pub size_bytes: i64,
}

/// Persist file bytes under a fresh, server-generated name.
pub fn store_file(assignment_id: Uuid, bytes: &[u8]) -> Result<StoredFile> {
    if bytes.is_empty() {
        return Err(anyhow!("file content must not be empty"));
    }
    if bytes.len() > max_file_bytes() {
        return Err(anyhow!(
            "file of {} bytes exceeds the {} byte limit",
            bytes.len(),
            max_file_bytes()
        ));
    }
    let dir = assignment_dir(assignment_id)?;
    let stored_name = Uuid::new_v4().to_string();
    fs::write(dir.join(&stored_name), bytes)?;
    Ok(StoredFile {
        stored_name,
        content_hash: sha256_hex(bytes),
        size_bytes: bytes.len() as i64,
    })
}

/// Read a previously stored file, verifying it lives inside the assignment
/// directory (defense in depth against a compromised stored name).
pub fn read_file(assignment_id: Uuid, stored_name: &str) -> Result<Vec<u8>> {
    let dir = assignment_dir(assignment_id)?;
    let canonical_dir = dir.canonicalize()?;
    let canonical = dir.join(stored_name).canonicalize()?;
    if !canonical.starts_with(&canonical_dir) {
        return Err(anyhow!("stored file path escapes the storage directory"));
    }
    Ok(fs::read(&canonical)?)
}

/// Delete a stored file (idempotent).
pub fn delete_file(assignment_id: Uuid, stored_name: &str) -> Result<()> {
    let dir = assignment_dir(assignment_id)?;
    let path = dir.join(stored_name);
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

/// Helper for tests: run `f` with a temporary storage directory.
#[cfg(test)]
fn with_temp_dir(f: impl FnOnce(&std::path::Path)) {
    let dir = std::env::temp_dir().join(format!("elupane-assign-test-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    // SAFETY: setting a test-only env var is sound in a single-threaded test.
    unsafe { std::env::set_var("ASSIGNMENTS_STORAGE_DIR", &dir) };
    f(&dir);
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal_names() {
        assert!(sanitize_filename("../../etc/passwd").is_err());
        assert!(sanitize_filename("..\\..\\windows\\win.ini").is_err());
        assert!(sanitize_filename("a/b.txt").is_err());
        assert!(sanitize_filename("a\\b.txt").is_err());
        assert!(sanitize_filename("").is_err());
        assert!(sanitize_filename("file\0name").is_err());
    }

    #[test]
    fn accepts_plain_names() {
        assert_eq!(sanitize_filename("essay.pdf").unwrap(), "essay.pdf");
        assert_eq!(
            sanitize_filename("my final report (v2).docx").unwrap(),
            "my final report (v2).docx"
        );
    }

    #[test]
    fn store_read_delete_round_trip() {
        with_temp_dir(|_dir| {
            let id = Uuid::new_v4();
            let bytes = b"hello world".to_vec();

            let stored = store_file(id, &bytes).unwrap();
            assert_eq!(stored.size_bytes, 11);
            assert_eq!(stored.content_hash, sha256_hex(&bytes));

            let read_back = read_file(id, &stored.stored_name).unwrap();
            assert_eq!(read_back, bytes);

            delete_file(id, &stored.stored_name).unwrap();
            assert!(read_file(id, &stored.stored_name).is_err());
        });
    }

    #[test]
    fn rejects_empty_and_oversized() {
        with_temp_dir(|_dir| {
            let id = Uuid::new_v4();
            assert!(store_file(id, &[]).is_err());

            // SAFETY: test-only env mutation in a single-threaded test.
            unsafe { std::env::set_var("ASSIGNMENTS_MAX_FILE_BYTES", "10") };
            assert!(store_file(id, b"this is more than ten bytes").is_err());
            unsafe { std::env::remove_var("ASSIGNMENTS_MAX_FILE_BYTES") };
        });
    }

    #[test]
    fn stored_names_are_server_generated_uuids() {
        with_temp_dir(|_dir| {
            let id = Uuid::new_v4();
            let a = store_file(id, b"aaa").unwrap();
            let b = store_file(id, b"bbb").unwrap();
            assert_ne!(a.stored_name, b.stored_name);
            // Both are valid UUIDs (and therefore path-safe).
            assert!(Uuid::parse_str(&a.stored_name).is_ok());
            assert!(Uuid::parse_str(&b.stored_name).is_ok());
        });
    }
}

