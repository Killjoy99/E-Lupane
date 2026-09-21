CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS notifications (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,
    type            TEXT NOT NULL,
    title           TEXT NOT NULL,
    body            TEXT NOT NULL DEFAULT '',
    data            JSONB NOT NULL DEFAULT '{}',
    read            BOOLEAN NOT NULL DEFAULT false,
    read_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications (user_id, created_at DESC);

DO $$
BEGIN
    ALTER TABLE notifications DROP CONSTRAINT IF EXISTS chk_notifications_type;
    ALTER TABLE notifications
        ADD CONSTRAINT chk_notifications_type
        CHECK (type IN ('enrollment', 'lesson', 'assignment_created', 'assignment_deadline', 'assignment_graded', 'quiz_available', 'exam_scheduled', 'exam_results', 'message', 'announcement', 'certificate', 'payment', 'support'));
END $$;

CREATE TABLE IF NOT EXISTS notification_preferences (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,
    notification_type TEXT NOT NULL,
    channel         TEXT NOT NULL,
    enabled         BOOLEAN NOT NULL DEFAULT true,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, notification_type, channel)
);

CREATE TABLE IF NOT EXISTS notification_deliveries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    channel         TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    error_message   TEXT,
    delivered_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_notification_deliveries_notification ON notification_deliveries (notification_id);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_notification_deliveries_channel'
    ) THEN
        ALTER TABLE notification_deliveries
            ADD CONSTRAINT chk_notification_deliveries_channel
            CHECK (channel IN ('in_app', 'email', 'push'));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_notification_deliveries_status'
    ) THEN
        ALTER TABLE notification_deliveries
            ADD CONSTRAINT chk_notification_deliveries_status
            CHECK (status IN ('pending', 'sent', 'failed'));
    END IF;
END $$;
