-- Support plugin: help-desk tickets, ticket threads, and an FAQ / knowledge base.
--
-- Idempotent raw SQL (IF NOT EXISTS / guarded), matching the other plugins'
-- style in the shared `elupane` database.

CREATE TABLE IF NOT EXISTS support_tickets (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject     TEXT NOT NULL,
    category    TEXT NOT NULL DEFAULT 'general',
    priority    TEXT NOT NULL DEFAULT 'normal'
                CHECK (priority IN ('low','normal','high','urgent')),
    status      TEXT NOT NULL DEFAULT 'open'
                CHECK (status IN ('open','in_progress','resolved','closed')),
    requester_id UUID NOT NULL,
    assignee_id UUID,
    course_id   UUID,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_support_tickets_requester ON support_tickets (requester_id);
CREATE INDEX IF NOT EXISTS idx_support_tickets_assignee  ON support_tickets (assignee_id);
CREATE INDEX IF NOT EXISTS idx_support_tickets_status    ON support_tickets (status);

-- The ticket thread (support messages).
CREATE TABLE IF NOT EXISTS support_messages (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id   UUID NOT NULL REFERENCES support_tickets(id) ON DELETE CASCADE,
    author_id   UUID NOT NULL,
    body        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_support_messages_ticket ON support_messages (ticket_id);

-- FAQ / knowledge base articles.
CREATE TABLE IF NOT EXISTS support_faq (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title       TEXT NOT NULL,
    body        TEXT NOT NULL DEFAULT '',
    category    TEXT NOT NULL DEFAULT 'general',
    created_by  UUID NOT NULL,
    published   BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_support_faq_published ON support_faq (published);
