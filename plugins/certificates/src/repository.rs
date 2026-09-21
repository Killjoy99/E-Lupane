//! Database repository for certificates plugin.

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::CertificateView;
use crate::models::Certificate;

/// Insert a new certificate record.
pub async fn insert_certificate(
    pool: &PgPool,
    certificate_code: &str,
    student_id: Uuid,
    course_id: Uuid,
    course_name: &str,
    student_name: &str,
    pdf_file_id: Option<Uuid>,
) -> anyhow::Result<Certificate> {
    let row = sqlx::query_as!(
        CertificateRow,
        r#"
        INSERT INTO certificates (certificate_code, student_id, course_id, course_name, student_name, pdf_file_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, certificate_code, student_id, course_id, course_name, student_name, completion_date, pdf_file_id, created_at
        "#,
        certificate_code,
        student_id,
        course_id,
        course_name,
        student_name,
        pdf_file_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

/// Check if certificate already exists for student+course.
pub async fn find_existing(
    pool: &PgPool,
    student_id: Uuid,
    course_id: Uuid,
) -> anyhow::Result<Option<Certificate>> {
    let row = sqlx::query_as!(
        CertificateRow,
        r#"
        SELECT id, certificate_code, student_id, course_id, course_name, student_name, completion_date, pdf_file_id, created_at
        FROM certificates
        WHERE student_id = $1 AND course_id = $2
        "#,
        student_id,
        course_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.into()))
}

/// Get certificate by ID.
pub async fn get_by_id(pool: &PgPool, certificate_id: Uuid) -> anyhow::Result<Option<Certificate>> {
    let row = sqlx::query_as!(
        CertificateRow,
        r#"
        SELECT id, certificate_code, student_id, course_id, course_name, student_name, completion_date, pdf_file_id, created_at
        FROM certificates
        WHERE id = $1
        "#,
        certificate_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.into()))
}

/// Get certificate by code.
pub async fn get_by_code(pool: &PgPool, certificate_code: &str) -> anyhow::Result<Option<Certificate>> {
    let row = sqlx::query_as!(
        CertificateRow,
        r#"
        SELECT id, certificate_code, student_id, course_id, course_name, student_name, completion_date, pdf_file_id, created_at
        FROM certificates
        WHERE certificate_code = $1
        "#,
        certificate_code,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.into()))
}

/// List certificates with pagination.
pub async fn list_certificates(
    pool: &PgPool,
    student_id: Option<Uuid>,
    course_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> anyhow::Result<(Vec<CertificateView>, i64)> {
    let rows = sqlx::query_as!(
        CertificateViewRow,
        r#"
        SELECT id, certificate_code, student_id, course_id, course_name, student_name, completion_date
        FROM certificates
        WHERE ($1::uuid IS NULL OR student_id = $1) AND ($2::uuid IS NULL OR course_id = $2)
        ORDER BY completion_date DESC
        LIMIT $3 OFFSET $4
        "#,
        student_id,
        course_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM certificates
        WHERE ($1::uuid IS NULL OR student_id = $1) AND ($2::uuid IS NULL OR course_id = $2)
        "#,
        student_id,
        course_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let certificates = rows.into_iter().map(|r| r.into()).collect();
    Ok((certificates, total))
}

// Internal row types

#[derive(sqlx::FromRow)]
struct CertificateRow {
    id: Uuid,
    certificate_code: String,
    student_id: Uuid,
    course_id: Uuid,
    course_name: String,
    student_name: String,
    completion_date: chrono::DateTime<chrono::Utc>,
    pdf_file_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<CertificateRow> for Certificate {
    fn from(row: CertificateRow) -> Self {
        Certificate {
            id: row.id,
            certificate_code: row.certificate_code,
            student_id: row.student_id,
            course_id: row.course_id,
            course_name: row.course_name,
            student_name: row.student_name,
            completion_date: row.completion_date,
            pdf_file_id: row.pdf_file_id,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CertificateViewRow {
    id: Uuid,
    certificate_code: String,
    student_id: Uuid,
    course_id: Uuid,
    course_name: String,
    student_name: String,
    completion_date: chrono::DateTime<chrono::Utc>,
}

impl From<CertificateViewRow> for CertificateView {
    fn from(row: CertificateViewRow) -> Self {
        CertificateView {
            id: row.id,
            certificate_code: row.certificate_code,
            student_id: row.student_id,
            course_id: row.course_id,
            course_name: row.course_name,
            student_name: row.student_name,
            completion_date: row.completion_date,
            pdf_url: None,
        }
    }
}
