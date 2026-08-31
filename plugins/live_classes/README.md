# Live Classes Plugin (`com.elupane.live_classes`)

Live learning sessions — scheduling, attendance, chat persistence and session
resources — per `AGENTS/live_classes.md`. Designed around a `provider`
integration field so `webrtc` (in-app, no external dependency), `jitsi`,
`zoom` and `teams` can all be scheduled without changing this plugin's course
logic; only the frontend's join flow branches on `provider`.

## What it does

* Schedule / update / cancel a session for a course (instructor/admin only —
  `services::is_manager`, mirroring the `enrollments` plugin's owner/
  course-instructor/admin check against the shared `courses` tables).
* Enforce the course relationship for each session (`live_sessions.course_id` →
  `courses.id`) so lesson scheduling remains tied to the owning course.
* Start / end a session (`scheduled → live → ended`, or `→ cancelled`).
* Join: validates the caller is a manager **or** has an *active* enrollment
  in the session's course (`repository::has_active_enrollment`, deferring to
  `true` if the enrollments plugin hasn't initialized its tables yet — same
  pattern the enrollments plugin uses for `course_management`). Records an
  attendance row and returns a signaling path (`/ws/live/{id}`) for
  `webrtc`-provider sessions.
* Leave: stamps `left_at` / `duration_seconds` on the attendance row.
* Attendance roster (managers only).
* Chat: persists messages (`live_chat_messages`); read back via
  `live.chat.history`. Real-time delivery is a separate concern — see the
  frontend's WebSocket relay.
* Session resources (links, whiteboard snapshots, recording pointers) and
  recording metadata (this plugin does not host recording infrastructure
  itself, only a pointer to wherever a recording ends up).

## Ops (Redis messaging)

`live.schedule` · `.update` · `.list` · `.get` · `.join` · `.leave` · `.start`
· `.end` · `.cancel` · `.attendance.list` · `.chat.send` · `.chat.history` ·
`.resource.attach` · `.resource.list` · `.recording.save`

## Files

* `src/authz.rs` — JWT verification (shared secret with the auth plugin)
* `src/dto.rs` — request/response payloads
* `src/models.rs` — SQLx row types
* `src/repository.rs` — data access + cross-plugin helpers (course ownership,
  instructor check, active-enrollment check against the shared tables)
* `src/services.rs` — business logic and authorization
* `src/handlers.rs` — op dispatch
* `migrations/0007_live_classes.sql` — schema: `live_sessions`,
  `live_attendance`, `live_chat_messages`, `live_session_resources`,
  `live_recordings`

## Real-time video/chat/screen-share/whiteboard

This plugin intentionally does **not** implement video conferencing itself
(per `AGENTS/live_classes.md`). For the `webrtc` provider, real-time transport
is a plain WebSocket exposed directly by the frontend's Axum server
(`/ws/live/{session_id}`, see `plugins/frontend/src/live_signaling.rs`), which
relays WebRTC signaling, chat and whiteboard strokes between participants in a
full-mesh topology. This plugin only owns the durable session/attendance/chat
state and authorization. See `plugins/frontend/README.md` for the full
real-time design and its known scaling limitation (single-instance relay).

## This session

* New plugin, created from scratch, modeled on the `enrollments` plugin's
  structure and conventions (JWT auth, idempotent raw-SQL migrations,
  Redis command/reply dispatch).
* `cargo build -p live_classes` and `cargo test -p live_classes` pass; the
  whole workspace (`cargo build --workspace` / `cargo test --workspace`)
  still builds and passes with this plugin added.
