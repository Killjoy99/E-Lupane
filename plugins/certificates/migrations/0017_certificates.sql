-- Certificates plugin: course completion certificates
-- Migration 0017

CREATE TABLE IF NOT EXISTS certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    certificate_code VARCHAR(16) UNIQUE NOT NULL,
    student_id UUID NOT NULL,
    course_id UUID NOT NULL,
    course_name TEXT NOT NULL,
    student_name TEXT NOT NULL,
    completion_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    pdf_file_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_certificates_student ON certificates (student_id);
CREATE INDEX IF NOT EXISTS idx_certificates_course ON certificates (course_id);
CREATE INDEX IF NOT EXISTS idx_certificates_code ON certificates (certificate_code);
CREATE UNIQUE INDEX IF NOT EXISTS idx_certificates_unique ON certificates (student_id, course_id);
