-- Media plugin: file storage metadata (local filesystem)
-- Migration 0016

CREATE TABLE IF NOT EXISTS media_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    course_id UUID,
    uploaded_by UUID NOT NULL,
    storage_key TEXT NOT NULL,
    etag TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'complete', 'deleted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_media_files_course ON media_files (course_id) WHERE status != 'deleted';
CREATE INDEX IF NOT EXISTS idx_media_files_uploader ON media_files (uploaded_by) WHERE status != 'deleted';
CREATE INDEX IF NOT EXISTS idx_media_files_status ON media_files (status);
CREATE INDEX IF NOT EXISTS idx_media_files_created ON media_files (created_at DESC);
