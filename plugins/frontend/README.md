# E-Lupane Frontend (Dioxus 0.7 Fullstack)

The Web UI for E-Lupane, built with **Dioxus 0.7**, served by an **Axum**
fullstack server and backed by **SQLx** (PostgreSQL).

## Stack

| Concern    | Technology                                        |
|------------|---------------------------------------------------|
| UI         | [Dioxus](https://dioxuslabs.com) 0.7 (fullstack)   |
| Web server | [Axum](https://github.com/tokio-rs/axum) (0.8)      |
| Database   | [SQLx](https://github.com/launchbadge/sqlx) (PostgreSQL) |

> Note: Dioxus 0.7's fullstack server integrates with **Axum** (there is no
> Actix integration). The rest of the E-Lupane workspace still uses Actix for
> its gateway, but this frontend uses Axum + SQLx + Dioxus together.

## Serve on port 8090

```powershell
cd frontend
dx serve --port 8090
```

`dx` builds and runs:
* the native **server** (feature `server`) — Axum + SQLx, bound per the
  `PORT`/`IP` env vars (dx sets these from `--port`);
* the **client** (feature `web`) — the WASM Dioxus app.

Then open http://localhost:8090 — click the buttons to call the Dioxus server
functions (`/api/health`, `/api/courses`), which run SQLx queries on the Axum
server.

## Setup the database

1. Have PostgreSQL running and create the `elupane` database.
2. Set `DATABASE_URL` (see `.env`) so the server can connect.
3. Run the E-Lupane migrations to create the `courses` table.

If PostgreSQL is unreachable the app still serves — `/api/health` just reports
`database: "unavailable"`.

## Files

* `src/main.rs` — entry point (server uses `dioxus::serve` + `dioxus::server::router`)
* `src/server_functions.rs` — Dioxus server functions (Axum endpoints)
* `src/db.rs` — SQLx connection pool (server-only)
* `src/dashboard.rs` — app shell: role routing, top bar, sidebar, login/session
* `src/assignments_ui.rs` — live assignment components (student + instructor pages)
* `src/courses_ui.rs` — live course components (catalogue + course management)
* `src/ui_helpers.rs` — small shared UI helpers (`fmt_dt`, file-to-base64)
* `src/mock_data.rs` — demo/mock data for the role pages
* `src/mock_pages.rs` — demo/mock role pages (student/lecturer/developer/admin)
* `src/assignments_types.rs` — shared assignment view/input types (client + server)
* `src/course_types.rs` — shared course view/input types + status helpers (client + server)
* `src/assignments_bridge.rs` — server-only Redis RPC bridge to the assignments
  plugin (`com.elupane.assignments`)
* `src/course_bridge.rs` — server-only Redis RPC bridge to the
  course_management plugin (`com.elupane.course_management`)
* `src/auth_bridge.rs` — server-only Redis RPC bridge to the auth plugin
* `src/dashboard.rs` — role-aware dashboard (Student / Lecturer / Developer / Admin)

## Courses

The dashboard's course pages are backed by the `course_management` plugin:

| Role     | Page          | What you can do |
|----------|---------------|-----------------|
| Student  | **My Courses** | Browse the published catalogue, open a course to read its description, objectives, prerequisites, audience and tags |
| Lecturer | **My Courses** | Create / edit courses (code, title, description, category, subject, level, language, difficulty, duration, objectives, prerequisites, tags), preview, publish / unpublish / submit for review, archive, duplicate, delete |
| Admin    | **Courses**    | Same management UI; admins can force any status transition |

Server functions (proxying to the course_management plugin):

* `POST /api/courses/list` · `/get` · `/create` · `/update` · `/delete`
* `POST /api/courses/archive` · `/duplicate` · `/preview` · `/status`
* `POST /api/courses/publish` · `/unpublish`

The **assignment editor** also lists the instructor's courses so assignments
can be linked to the course they belong to (the plugin validates that the
instructor manages the course before accepting the link).

## Assignments

The dashboard has live, plugin-backed assignment pages for three roles:

| Role     | Page                  | What you can do |
|----------|-----------------------|-----------------|
| Student  | **Assignments**       | See published briefs, open one, submit text / file / link, resubmit, view attempt status & grade |
| Lecturer | **Assignments**       | Create / edit / delete assignments, attach resource files, review submissions, grade (with late-penalty feedback) |
| Admin    | **Assignments**       | Same management UI as lecturers (admins are instructors on the plugin) |

The pages call server functions that proxy over the Redis messaging layer to the
assignments plugin (JWT from the auth session is forwarded on every call):

* `POST /api/assignments/list` · `/get` · `/create` · `/update` · `/delete`
* `POST /api/assignments/attach` · `/resource/download`
* `POST /api/assignments/submit` · `/status`
* `POST /api/assignments/grade` · `/grade/get` · `/history`

Files are read in the browser as base64 (25 MiB limit enforced server-side) and
stored by the plugin under `data/assignments/<assignment_id>/<uuid>`.

## Live Classes

Real-time video classroom, backed by the new `live_classes` plugin for durable
state and by this frontend's own WebSocket relay for real-time transport:

| Role                | Page              | What you can do |
|---------------------|-------------------|-----------------|
| Student              | **Live Classes** | Browse a course's sessions, join a live session (video, chat, screen share, whiteboard) |
| Lecturer / Admin     | **Live Classes** | Schedule sessions (in-app WebRTC or an external Jitsi/Zoom/Teams link), start/end/cancel, join as host |

* `POST /api/live/list` · `/get` · `/schedule` · `/join` · `/leave` · `/start` ·
  `/end` · `/cancel` · `/attendance` · `/chat/history` · `/chat/send` ·
  `/resource/attach` · `/resource/list` — proxy to the `live_classes` plugin.
* `GET /ws/live/{session_id}?token=&role=&attendance_id=` — a plain WebSocket
  (not Redis RPC) that relays, for `webrtc`-provider sessions only:
  * WebRTC offer/answer/ICE candidates (full-mesh peer connections, one per
    other participant);
  * chat messages (also persisted via `/api/live/chat/send`);
  * whiteboard stroke events (ephemeral — draw ops are broadcast, not stored).

  See `src/live_signaling.rs` for the server-side relay and `src/live_room.rs`
  (`LIVE_ROOM_JS`) for the browser-side WebRTC/canvas/whiteboard logic. This
  relay lives in-process (per frontend instance) — running multiple frontend
  replicas behind a load balancer would need it to fan out over Redis Pub/Sub
  instead; tracked as a known limitation.
* `src/live_types.rs` — shared view types (client + server)
* `src/live_bridge.rs` — server-only Redis RPC bridge to `com.elupane.live_classes`
* `src/live_classes_ui.rs` — schedule/browse/join UI
* `src/live_room.rs` — the live classroom component (video grid, chat, whiteboard controls)
* `src/mcp_server.rs` — server-only Model Context Protocol endpoint (Streamable
  HTTP) exposing **every plugin's operations** as AI-callable tools

## MCP endpoint (AI clients)

`POST /api/mcp` speaks the [Model Context Protocol](https://modelcontextprotocol.io)
(Streamable HTTP transport, JSON-RPC 2.0) and exposes the system's operations as
AI-callable **tools**, so an external agent (Claude Desktop, Cline, VS Code, …)
can read and act on E-Lupane on behalf of the signed-in user.

Two kinds of tools are served:

1. **Typed notification tools** — hand-written JSON schemas:

| Tool                              | Plugin op                      | Notes                |
|-----------------------------------|--------------------------------|----------------------|
| `notifications_list`              | `notifications.list`           | signed-in user only  |
| `notifications_unread`            | `notifications.unread`         | signed-in user only  |
| `notifications_mark_read`         | `notifications.mark_read`      | mutates state        |
| `notifications_mark_all_read`     | `notifications.mark_all_read`  | mutates state        |
| `notifications_send`              | `notifications.send`           | admin / lecturer     |
| `notifications_preferences_list`  | `notifications.preferences.list` | signed-in user only |
| `notifications_preferences_set`   | `notifications.preferences.set` | mutates state       |
| `notifications_deliveries`        | `notifications.deliveries`     | read-only            |

2. **Generic plugin-op tools** — one tool per operation of every other plugin
   (academics, analytics, assignments, auth, certificates, content, course
   management, enrollments, grades, institution management, learning progress,
   live classes, media, messaging, quizzes, search, support — ~200 tools).
   Names follow `<plugin>_<op with dots → underscores>` (e.g.
   `messaging_discussion_list`, `quizzes_attempts_start`,
   `enrollments_enrollment_enroll`). Their input schema is a free-form JSON
   object passed through to the plugin, which validates fields and returns a
   descriptive error the model can self-correct from.

* **Auth**: `Authorization: Bearer <auth-plugin JWT>` — the same session token
  the browser holds (`?token=` is also accepted). It is verified through
  `auth_bridge::verify` and forwarded to the plugin unchanged.
* **Credential injection**: the verified token is injected as `token` (or
  `admin_token` for the auth admin ops that expect it there); any `token` /
  `admin_token` supplied by the model is stripped first, so an agent can never
  present a different identity.
* **Scoping**: the *verified* user id is injected into every user-scoped
  notification payload, so an agent can never ask for another user's
  notifications or preferences. For the other plugins, each plugin's own authz
  (roles/ownership) still enforces access — the MCP layer adds authentication,
  not authorization.
* **Annotations**: `readOnlyHint` is set on read-shaped ops (`get`, `list`,
  `history`, search, …), and a DESTRUCTIVE warning is appended to the
  description of ops that delete/cancel/archive/revoke, so agents confirm with
  the user first.
* **Shape**: only `tools/*` is advertised; `GET` (server-initiated SSE) answers
  `405`, and no server-side session state is kept (`DELETE` is a no-op success).

Example MCP client configuration:

```json
{
  "mcpServers": {
    "elupane": {
      "type": "http",
      "url": "http://127.0.0.1:8090/api/mcp",
      "headers": { "Authorization": "Bearer <session token>" }
    }
  }
}
```

Implementation: `src/mcp_server.rs`. Typed tools live in `TOOLS` +
`build_payload`; new plugin ops only need one line appended to `GENERIC_TOOLS`
(the `(plugin id, short name, op)` table) — names, descriptions, annotations,
dispatch and credential injection are derived automatically.

## Recent changes

* Fixed a syntax error in `auth_bridge.rs` (missing brace in `verify()`, a
  duplicated `ACCOUNT_TYPES` constant and stray trailing code) that broke the
  whole crate's compilation.
* Fixed a pre-existing `crate::` path bug in `server_functions::auth_roles`
  (`auth_bridge::ACCOUNT_TYPES` → `crate::auth_bridge::ACCOUNT_TYPES`).
* Added the full **Live Classes** feature end to end: plugin RPC bridge,
  shared types, real-time WebSocket signaling relay, scheduling/browse UI and
  the in-app video/chat/screen-share/whiteboard room (see above).

