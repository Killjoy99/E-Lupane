-- Enrollments plugin: course enrollments, history and invitations.
--
-- Applied with idempotent raw SQL so it is safe to run alongside the other
-- plugins' migrations in the shared `_sqlx_migrations`-free database.

CREATE TABLE IF NOT EXISTS course_enrollments (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id    UUID NOT NULL,
    student_id   UUID NOT NULL,
    state        TEXT NOT NULL DEFAULT 'pending'
                 CHECK (state IN ('pending','active','suspended','completed','expired','cancelled')),
    -- How the enrollment was created: self | instructor | admin | bulk | invite
    source       TEXT NOT NULL DEFAULT 'self',
    -- Who created / last modified the enrollment (NULL for self-service).
    enrolled_by  UUID,
    -- Optional expiry deadline for the enrollment (active enrollments past
    -- this moment are swept into the 'expired' state).
    expires_at   TIMESTAMPTZ,
    approved_by  UUID,
    approved_at  TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    suspended_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- One row per (course, student): re-enrollment reuses and rewinds the row,
    -- every state change is recorded in enrollment_history.
    CONSTRAINT uq_course_student UNIQUE (course_id, student_id)
);

CREATE INDEX IF NOT EXISTS idx_enrollments_course ON course_enrollments (course_id);
CREATE INDEX IF NOT EXISTS idx_enrollments_student ON course_enrollments (student_id);
CREATE INDEX IF NOT EXISTS idx_enrollments_state ON course_enrollments (state);

CREATE TABLE IF NOT EXISTS enrollment_history (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    enrollment_id UUID NOT NULL REFERENCES course_enrollments(id) ON DELETE CASCADE,
    from_state    TEXT,
    to_state      TEXT NOT NULL,
    changed_by    UUID,
    reason        TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_enrollment_history_enrollment
    ON enrollment_history (enrollment_id);

CREATE TABLE IF NOT EXISTS course_invites (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id  UUID NOT NULL,
    token      TEXT NOT NULL UNIQUE,
    created_by UUID NOT NULL,
    -- Optional cap on how many students may join with this invite.
    max_uses   INTEGER,
    uses       INTEGER NOT NULL DEFAULT 0,
    -- Optional deadline after which the invite stops working.
    expires_at TIMESTAMPTZ,
    active     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_course_invites_course ON course_invites (course_id);