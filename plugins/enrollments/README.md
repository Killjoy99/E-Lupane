# Enrollments Plugin (`com.elupane.enrollments`)

Course enrollment lifecycle, rosters, invitations, history and content-access
enforcement, per `AGENTS/enrollment.md`.

## What it does

* Student self-enrollment (lands in `pending`), instructor/admin enrollment,
  bulk enrollment, invite-link joins.
* Enrollment states: `pending → active → suspended/completed/expired/cancelled`,
  with re-enrollment support (see `dto::can_transition`).
* Approval, suspension, cancellation, completion — enforced server-side via
  `services::change_state`, gated by `services::is_manager` (admin, course
  owner, or course instructor).
* Automatic expiry sweep every 60s (`services::sweep_expired`, run from
  `main.rs`'s event loop) moves `active` enrollments past `expires_at` to
  `expired`.
* Append-only enrollment history (`enrollment_history` table) for every state
  transition.
* Course-content access checks (`enrollment.access` op) so other plugins
  (assignments, live_classes, ...) can gate content behind an *active*
  enrollment.

## Ops (Redis messaging)

`enrollment.self` · `.enroll` · `.bulk` · `.approve` · `.suspend` · `.cancel` ·
`.complete` · `.reenroll` · `.roster` · `.mine` · `.get` · `.history` ·
`.access` · `.invite.create` · `.invite.list` · `.invite.revoke` · `.join`

## Files

* `src/authz.rs` — JWT verification + pure permission-rule helpers (unit tested)
* `src/dto.rs` — request/response payloads + the enrollment state machine
* `src/models.rs` — SQLx row types
* `src/repository.rs` — data access (`course_enrollments`, `enrollment_history`, `course_invites`)
* `src/services.rs` — business logic (the source of truth for authorization —
  see note below)
* `src/handlers.rs` — op dispatch
* `migrations/0004_enrollments.sql` — schema (idempotent raw SQL)

## Known housekeeping note

`services.rs` implements its own manager/authorization checks directly against
the shared `courses`/`course_instructors` tables (`is_manager`, inline checks
in `change_state`, etc.) rather than calling the pure rule functions in
`authz.rs` (`can_manage_roster`, `can_enroll_other`, `can_access_content`,
`can_change_enrollment`, `Caller`). Both implement the same rules; the
`authz.rs` versions are effectively an earlier design that predates the
DB-backed checks in `services.rs` and are currently unused (they still compile
and are covered by their own unit tests). Left as-is this session since
`services.rs`'s enforcement is correct and exercised by tests — worth
deleting or consolidating in a follow-up pass.

## This session

* Verified the plugin builds and its test suite passes (`cargo test -p
  enrollments`) as part of a full-workspace check; no functional changes were
  made here. The compile error blocking the whole build was in the frontend's
  `auth_bridge.rs`, not in this plugin (see `plugins/frontend/README.md`).
