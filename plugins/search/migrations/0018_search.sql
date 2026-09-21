-- Search plugin: full-text search indexes
-- Migration 0018

-- Add search_vector column to courses if not exists
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'courses' AND column_name = 'search_vector') THEN
        ALTER TABLE courses ADD COLUMN search_vector tsvector;
    END IF;
END$$;

-- Add search_vector column to content_items if not exists
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'content_items' AND column_name = 'search_vector') THEN
        ALTER TABLE content_items ADD COLUMN search_vector tsvector;
    END IF;
END$$;

-- Create GIN indexes for full-text search
CREATE INDEX IF NOT EXISTS idx_courses_search ON courses USING GIN (search_vector);
CREATE INDEX IF NOT EXISTS idx_content_search ON content_items USING GIN (search_vector);

-- Create triggers to auto-update search vectors
CREATE OR REPLACE FUNCTION courses_search_vector_update() RETURNS trigger AS $$
BEGIN
    NEW.search_vector := to_tsvector('english', coalesce(NEW.title, '') || ' ' || coalesce(NEW.description, ''));
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION content_search_vector_update() RETURNS trigger AS $$
BEGIN
    NEW.search_vector := to_tsvector('english', coalesce(NEW.title, '') || ' ' || coalesce(NEW.body, ''));
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Drop existing triggers if they exist, then create them
DROP TRIGGER IF EXISTS courses_search_trigger ON courses;
CREATE TRIGGER courses_search_trigger
    BEFORE INSERT OR UPDATE ON courses
    FOR EACH ROW
    EXECUTE FUNCTION courses_search_vector_update();

DROP TRIGGER IF EXISTS content_search_trigger ON content_items;
CREATE TRIGGER content_search_trigger
    BEFORE INSERT OR UPDATE ON content_items
    FOR EACH ROW
    EXECUTE FUNCTION content_search_vector_update();

-- Update existing rows
UPDATE courses SET search_vector = to_tsvector('english', coalesce(title, '') || ' ' || coalesce(description, ''));
UPDATE content_items SET search_vector = to_tsvector('english', coalesce(title, '') || ' ' || coalesce(body, ''));
