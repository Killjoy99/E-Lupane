-- Live classes plugin: scheduled sessions, attendance, chat, resources.
--
-- Applied with idempotent raw SQL, like the other plugins sharing the
-- `elupane` database. This is 0007 because auth owns 0001, assignments 0002,
-- course_management 0001-0004, enrollments 0004, institution_management 0005,
-- academics 0006.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Scheduled / live / ended learning sessions tied to a course.
CREATE TABLE IF NOT EXISTS live_sessions (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id      UUID NOT NULL,
    title          TEXT NOT NULL,
    description    TEXT NOT NULL DEFAULT '',
    host_id        UUID NOT NULL,
    -- provider backing this session's real-time transport: webrtc | jitsi | zoom | teams
    provider       TEXT NOT NULL DEFAULT 'webrtc',
    -- provider-specific join info (e.g. an external meeting URL). NULL for webrtc,
    -- which is served in-app via the frontend's signaling relay.
    external_url   TEXT,
    scheduled_start TIMESTAMPTZ NOT NULL,
    scheduled_end   TIMESTAMPTZ NOT NULL,
    started_at     TIMESTAMPTZ,
    ended_at       TIMESTAMPTZ,
    status         TEXT NOT NULL DEFAULT 'scheduled'
                   CHECK (status IN ('scheduled','live','ended','cancelled')),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_live_sessions_course ON live_sessions (course_id);
CREATE INDEX IF NOT EXISTS idx_live_sessions_status ON live_sessions (status);
CREATE INDEX IF NOT EXISTS idx_live_sessions_host ON live_sessions (host_id);

DO $$
BEGIN
    IF to_regclass('public.courses') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_live_sessions_course'
       ) THEN
        ALTER TABLE live_sessions
            ADD CONSTRAINT fk_live_sessions_course
            FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE;
    END IF;
END $$;

-- Attendance: one row per participant per session, tracking join/leave.
CREATE TABLE IF NOT EXISTS live_attendance (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id   UUID NOT NULL REFERENCES live_sessions(id) ON DELETE CASCADE,
    user_id      UUID NOT NULL,
    role         TEXT NOT NULL DEFAULT 'attendee' CHECK (role IN ('host', 'attendee')),
    joined_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    left_at      TIMESTAMPTZ,
    duration_seconds INTEGER,
    UNIQUE (session_id, user_id, joined_at)
);

CREATE INDEX IF NOT EXISTS idx_live_attendance_session ON live_attendance (session_id);
CREATE INDEX IF NOT EXISTS idx_live_attendance_user ON live_attendance (user_id);

-- Persisted chat history for a session (also relayed live over the frontend's
-- WebSocket signaling channel; this table is the durable record).
CREATE TABLE IF NOT EXISTS live_chat_messages (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id   UUID NOT NULL REFERENCES live_sessions(id) ON DELETE CASCADE,
    sender_id    UUID NOT NULL,
    body         TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_live_chat_session ON live_chat_messages (session_id, created_at);

-- Resources shared for a session (links; e.g. slides, notes, whiteboard export).
CREATE TABLE IF NOT EXISTS live_session_resources (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id   UUID NOT NULL REFERENCES live_sessions(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    url          TEXT NOT NULL,
    kind         TEXT NOT NULL DEFAULT 'link', -- link | whiteboard_snapshot | recording
    created_by   UUID NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_live_resources_session ON live_session_resources (session_id);

-- Recording metadata (the platform does not host recording infrastructure
-- itself; this stores a pointer to wherever the recording was persisted).
CREATE TABLE IF NOT EXISTS live_recordings (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id   UUID NOT NULL REFERENCES live_sessions(id) ON DELETE CASCADE,
    url          TEXT NOT NULL,
    duration_seconds INTEGER,
    size_bytes   BIGINT,
    created_by   UUID NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_live_recordings_session ON live_recordings (session_id);
