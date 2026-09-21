//! Data transfer objects for certificates plugin.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request to generate a completion certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCertificateRequest {
    pub course_id: Uuid,
    pub student_id: Option<Uuid>, // Admin can generate for others
}

/// Certificate view returned after generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateView {
    pub id: Uuid,
    pub certificate_code: String,
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub course_name: String,
    pub student_name: String,
    pub completion_date: chrono::DateTime<chrono::Utc>,
    pub pdf_url: Option<String>,
}

/// List certificates with optional filters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCertificatesRequest {
    pub student_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Certificate list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateListResponse {
    pub certificates: Vec<CertificateView>,
    pub total: i64,
}

/// Get a single certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCertificateRequest {
    pub certificate_id: Uuid,
}

/// Verify a certificate by its public code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyCertificateRequest {
    pub certificate_code: String,
}

/// Verification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub certificate: Option<CertificateView>,
    pub message: String,
}
