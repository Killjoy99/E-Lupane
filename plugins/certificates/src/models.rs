//! Domain models for certificates plugin.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents a completion certificate.
#[derive(Debug, Clone)]
pub struct Certificate {
    pub id: Uuid,
    pub certificate_code: String,
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub course_name: String,
    pub student_name: String,
    pub completion_date: DateTime<Utc>,
    pub pdf_file_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl Certificate {
    /// Generate a unique certificate code.
    pub fn generate_code() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let mut rng = rand::rng();
        (0..12)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}
