CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS discussion_topics (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id       UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    module_id       UUID,
    lesson_id       UUID,
    user_id         UUID NOT NULL,
    title           TEXT NOT NULL,
    body            TEXT NOT NULL DEFAULT '',
    pinned          BOOLEAN NOT NULL DEFAULT false,
    locked          BOOLEAN NOT NULL DEFAULT false,
    marked_answer_id UUID,
    status          TEXT NOT NULL DEFAULT 'open',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_discussion_topics_course ON discussion_topics (course_id);
CREATE INDEX IF NOT EXISTS idx_discussion_topics_user ON discussion_topics (user_id);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_discussion_topics_status'
    ) THEN
        ALTER TABLE discussion_topics
            ADD CONSTRAINT chk_discussion_topics_status
            CHECK (status IN ('open', 'closed', 'archived'));
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS discussion_replies (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    topic_id        UUID NOT NULL REFERENCES discussion_topics(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES discussion_replies(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL,
    body            TEXT NOT NULL DEFAULT '',
    is_answer       BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_discussion_replies_topic ON discussion_replies (topic_id);
CREATE INDEX IF NOT EXISTS idx_discussion_replies_parent ON discussion_replies (parent_id);

CREATE TABLE IF NOT EXISTS discussion_attachments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    topic_id        UUID REFERENCES discussion_topics(id) ON DELETE CASCADE,
    reply_id        UUID REFERENCES discussion_replies(id) ON DELETE CASCADE,
    file_name       TEXT NOT NULL,
    file_path       TEXT NOT NULL,
    mime_type       TEXT NOT NULL,
    size_bytes      BIGINT NOT NULL DEFAULT 0,
    uploaded_by     UUID NOT NULL,
    uploaded_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_discussion_attachments_topic ON discussion_attachments (topic_id);
CREATE INDEX IF NOT EXISTS idx_discussion_attachments_reply ON discussion_attachments (reply_id);

CREATE TABLE IF NOT EXISTS conversations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS conversation_participants (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL,
    last_read_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_conversation_participants_conversation ON conversation_participants (conversation_id);
CREATE INDEX IF NOT EXISTS idx_conversation_participants_user ON conversation_participants (user_id);

CREATE TABLE IF NOT EXISTS messages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_id       UUID NOT NULL,
    body            TEXT NOT NULL DEFAULT '',
    read_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages (conversation_id);
CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages (sender_id);

CREATE TABLE IF NOT EXISTS message_attachments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id      UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    file_name       TEXT NOT NULL,
    file_path       TEXT NOT NULL,
    mime_type       TEXT NOT NULL,
    size_bytes      BIGINT NOT NULL DEFAULT 0,
    uploaded_by     UUID NOT NULL,
    uploaded_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_message_attachments_message ON message_attachments (message_id);

CREATE TABLE IF NOT EXISTS discussion_reports (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    topic_id        UUID REFERENCES discussion_topics(id) ON DELETE CASCADE,
    reply_id        UUID REFERENCES discussion_replies(id) ON DELETE CASCADE,
    reporter_id     UUID NOT NULL,
    reason          TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'open',
    resolved_by     UUID,
    resolved_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_discussion_reports_topic ON discussion_reports (topic_id);
CREATE INDEX IF NOT EXISTS idx_discussion_reports_reply ON discussion_reports (reply_id);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_discussion_reports_status'
    ) THEN
        ALTER TABLE discussion_reports
            ADD CONSTRAINT chk_discussion_reports_status
            CHECK (status IN ('open', 'resolved', 'dismissed'));
    END IF;
END $$;
