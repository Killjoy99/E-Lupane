CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS analytics_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type      TEXT NOT NULL,
    user_id         UUID NOT NULL,
    course_id       UUID,
    resource_type   TEXT,
    resource_id     UUID,
    metadata        JSONB NOT NULL DEFAULT '{}',
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_analytics_events_user ON analytics_events (user_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_course ON analytics_events (course_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_type ON analytics_events (event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_events_occurred ON analytics_events (occurred_at);

CREATE TABLE IF NOT EXISTS analytics_aggregates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bucket          TEXT NOT NULL,
    bucket_start    TIMESTAMPTZ NOT NULL,
    bucket_end      TIMESTAMPTZ NOT NULL,
    user_id         UUID,
    course_id       UUID,
    metric          TEXT NOT NULL,
    value           NUMERIC NOT NULL DEFAULT 0,
    metadata        JSONB NOT NULL DEFAULT '{}',
    calculated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_analytics_aggregates_bucket ON analytics_aggregates (bucket, bucket_start, bucket_end);
CREATE INDEX IF NOT EXISTS idx_analytics_aggregates_course ON analytics_aggregates (course_id);
CREATE INDEX IF NOT EXISTS idx_analytics_aggregates_user ON analytics_aggregates (user_id);

CREATE TABLE IF NOT EXISTS analytics_reports (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_type     TEXT NOT NULL,
    format          TEXT NOT NULL,
    filters         JSONB NOT NULL DEFAULT '{}',
    data            JSONB NOT NULL DEFAULT '{}',
    file_path       TEXT,
    generated_by    UUID NOT NULL,
    generated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_analytics_reports_type ON analytics_reports (report_type);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_analytics_reports_format'
    ) THEN
        ALTER TABLE analytics_reports
            ADD CONSTRAINT chk_analytics_reports_format
            CHECK (format IN ('csv', 'pdf', 'xlsx'));
    END IF;
END $$;
