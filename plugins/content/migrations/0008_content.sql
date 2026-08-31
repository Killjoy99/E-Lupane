-- Content plugin: learning modules, lessons, and content items.
--
-- NOTE: version 0008 because auth owns 0001, assignments 0002,
-- course_management 0003, institution_management 0005, enrollments 0004,
-- academics 0006, live_classes 0007. This schema uses idempotent raw SQL
-- (IF NOT EXISTS + guarded ALTER), so it is safe to run alongside the
-- other plugins regardless of startup order.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Modules group lessons within a course.
CREATE TABLE IF NOT EXISTS content_modules (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id       UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    sort_order      INTEGER NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'draft',
    published_at    TIMESTAMPTZ,
    created_by      UUID NOT NULL,
    updated_by      UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_content_modules_course ON content_modules (course_id);
CREATE INDEX IF NOT EXISTS idx_content_modules_sort ON content_modules (course_id, sort_order);

-- Lessons belong to a module and are ordered within that module.
CREATE TABLE IF NOT EXISTS content_lessons (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id       UUID NOT NULL REFERENCES content_modules(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    sort_order      INTEGER NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'draft',
    published_at    TIMESTAMPTZ,
    created_by      UUID NOT NULL,
    updated_by      UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_content_lessons_module ON content_lessons (module_id);
CREATE INDEX IF NOT EXISTS idx_content_lessons_sort ON content_lessons (module_id, sort_order);

-- Content items are the leaf nodes: text, video, audio, PDF, Word, PowerPoint,
-- images, external links, or embedded content.
CREATE TABLE IF NOT EXISTS content_items (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lesson_id       UUID NOT NULL REFERENCES content_lessons(id) ON DELETE CASCADE,
    content_type    TEXT NOT NULL,
    title           TEXT NOT NULL DEFAULT '',
    body            TEXT NOT NULL DEFAULT '',
    url             TEXT NOT NULL DEFAULT '',
    file_id         UUID,
    mime_type       TEXT NOT NULL DEFAULT 'text/plain',
    file_size_bytes BIGINT,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_by      UUID NOT NULL,
    updated_by      UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_content_items_lesson ON content_items (lesson_id);
CREATE INDEX IF NOT EXISTS idx_content_items_sort ON content_items (lesson_id, sort_order);

-- File attachments for content items (PDF, Word, PowerPoint, images, video, audio).
-- Files are stored with a UUID so they are not guessable and can be served
-- through an authenticated endpoint rather than public URLs.
CREATE TABLE IF NOT EXISTS content_files (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id       UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    filename        TEXT NOT NULL,
    mime_type       TEXT NOT NULL,
    size_bytes      BIGINT NOT NULL,
    storage_path    TEXT NOT NULL,
    created_by      UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_content_files_course ON content_files (course_id);

-- Enforce valid content_type values.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'chk_content_items_type'
    ) THEN
        ALTER TABLE content_items
            ADD CONSTRAINT chk_content_items_type
            CHECK (content_type IN (
                'text', 'video', 'audio', 'pdf', 'word', 'powerpoint',
                'image', 'external_link', 'embedded'
            ));
    END IF;
END $$;

-- Enforce valid module/lesson status values.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'chk_content_modules_status'
    ) THEN
        ALTER TABLE content_modules
            ADD CONSTRAINT chk_content_modules_status
            CHECK (status IN ('draft', 'published', 'unpublished'));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'chk_content_lessons_status'
    ) THEN
        ALTER TABLE content_lessons
            ADD CONSTRAINT chk_content_lessons_status
            CHECK (status IN ('draft', 'published', 'unpublished'));
    END IF;
END $$;
