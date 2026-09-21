//! Business logic for certificates plugin.

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{
    CertificateListResponse, CertificateView, GenerateCertificateRequest, ListCertificatesRequest,
    VerificationResult,
};
use crate::models::Certificate;
use crate::repository;

/// Generate a completion certificate for a course.
pub async fn generate_certificate(
    pool: &PgPool,
    user_id: Uuid,
    req: &GenerateCertificateRequest,
) -> anyhow::Result<CertificateView> {
    // Determine the student (self or admin generating for another)
    let student_id = req.student_id.unwrap_or(user_id);

    // Check if certificate already exists
    if let Some(existing) = repository::find_existing(pool, student_id, req.course_id).await? {
        return Ok(CertificateView {
            id: existing.id,
            certificate_code: existing.certificate_code,
            student_id: existing.student_id,
            course_id: existing.course_id,
            course_name: existing.course_name,
            student_name: existing.student_name,
            completion_date: existing.completion_date,
            pdf_url: existing.pdf_file_id.map(|_| format!("/api/certificates/{}/pdf", existing.id)),
        });
    }

    // Get course and student info (simplified - in production, fetch from plugins)
    let course_name = get_course_name(pool, req.course_id).await?;
    let student_name = get_student_name(pool, student_id).await?;

    // Generate unique certificate code
    let certificate_code = Certificate::generate_code();

    // Create certificate record
    let certificate = repository::insert_certificate(
        pool,
        &certificate_code,
        student_id,
        req.course_id,
        &course_name,
        &student_name,
        None, // PDF generation deferred to media_service integration
    )
    .await?;

    Ok(CertificateView {
        id: certificate.id,
        certificate_code: certificate.certificate_code,
        student_id: certificate.student_id,
        course_id: certificate.course_id,
        course_name: certificate.course_name,
        student_name: certificate.student_name,
        completion_date: certificate.completion_date,
        pdf_url: None,
    })
}

/// List certificates with pagination.
pub async fn list_certificates(
    pool: &PgPool,
    user_id: Uuid,
    is_admin: bool,
    req: &ListCertificatesRequest,
) -> anyhow::Result<CertificateListResponse> {
    let limit = req.limit.unwrap_or(50).min(100);
    let offset = req.offset.unwrap_or(0);

    // Non-admins can only see their own certificates
    let student_id = if is_admin { req.student_id } else { Some(user_id) };

    let (certificates, total) = repository::list_certificates(pool, student_id, req.course_id, limit, offset).await?;

    Ok(CertificateListResponse { certificates, total })
}

/// Get a single certificate.
pub async fn get_certificate(
    pool: &PgPool,
    user_id: Uuid,
    is_admin: bool,
    certificate_id: &Uuid,
) -> anyhow::Result<CertificateView> {
    let certificate = repository::get_by_id(pool, *certificate_id).await?;

    let certificate = certificate.ok_or_else(|| anyhow::anyhow!("certificate not found"))?;

    // Verify ownership or admin
    if certificate.student_id != user_id && !is_admin {
        return Err(anyhow::anyhow!("access denied"));
    }

    Ok(CertificateView {
        id: certificate.id,
        certificate_code: certificate.certificate_code,
        student_id: certificate.student_id,
        course_id: certificate.course_id,
        course_name: certificate.course_name,
        student_name: certificate.student_name,
        completion_date: certificate.completion_date,
        pdf_url: certificate.pdf_file_id.map(|_| format!("/api/certificates/{}/pdf", certificate.id)),
    })
}

/// Verify a certificate by its public code.
pub async fn verify_certificate(
    pool: &PgPool,
    certificate_code: &str,
) -> anyhow::Result<VerificationResult> {
    let certificate = repository::get_by_code(pool, certificate_code).await?;

    match certificate {
        Some(cert) => {
            let view = CertificateView {
                id: cert.id,
                certificate_code: cert.certificate_code,
                student_id: cert.student_id,
                course_id: cert.course_id,
                course_name: cert.course_name,
                student_name: cert.student_name,
                completion_date: cert.completion_date,
                pdf_url: None,
            };
            Ok(VerificationResult {
                valid: true,
                certificate: Some(view),
                message: "Certificate is valid".to_string(),
            })
        }
        None => Ok(VerificationResult {
            valid: false,
            certificate: None,
            message: "Certificate not found".to_string(),
        }),
    }
}

// Helper functions (simplified - in production, call other plugins via RPC)

async fn get_course_name(pool: &PgPool, course_id: Uuid) -> anyhow::Result<String> {
    // The courses table stores its display name in `title`.
    let title: Option<String> = sqlx::query_scalar!(
        "SELECT title FROM courses WHERE id = $1",
        course_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(title
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown Course".to_string()))
}

async fn get_student_name(pool: &PgPool, student_id: Uuid) -> anyhow::Result<String> {
    let name = sqlx::query_scalar!(
        "SELECT COALESCE(NULLIF(full_name, ''), username) FROM users WHERE id = $1",
        student_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(name
        .flatten()
        .unwrap_or_else(|| "Unknown Student".to_string()))
}
