# High-Priority Missing Features Implementation Plan

## Goal

Implement the 6 highest-value missing features for the E-Lupane e-learning system:
1. Wire up the `messaging` plugin (discussions + direct messaging)
2. Implement `media_service` (S3/GCS-compatible file uploads)
3. Build `certificates` (completion certificates)
4. Stand up `search` (full-text course/content search)
5. Clean up duplicate stubs (`courses`, `files`, `discussions`)
6. Build `support` (user support: help-desk tickets + FAQ/knowledge base)

These are all production-blocking or high-ROI gaps identified in the codebase audit.

---

## Current State

| Feature | Status | Gap |
|---------|--------|-----|
| `messaging` plugin | DB + services + models exist, `main.rs` is heartbeat, **no `handlers.rs`** | RPC not exposed; discussions/DM dead code |
| `media_service` | Stub returning `"Hello, world!"` | Content uploads/downloads broken in production |
| `certificates` | Heartbeat stub | No completion certificates despite `learning_progress` tracking |
| `search` plugin | Heartbeat stub | No full-text search |
| `search_service` | Stub | No search backend |
| Duplicate stubs | `courses`, `files`, `discussions` | Name collisions with real plugins |
| `support` plugin | Does not exist | No help desk / FAQ; users have no channel to ask questions or report issues |

---

## Design Decisions (Locked)

1. **messaging dispatch**: Use the same `PluginClient` Redis RPC pattern as all other plugins. Announce `messaging.discussion.*` and `messaging.conversation.*` operations.
2. **messaging scope v1**: Discussions only (topics + replies). Direct messaging (conversations) is modeled in DB but deferred to v2 to keep scope tight.
3. **media_service**: MinIO/S3-compatible API only. No local filesystem storage. Signed URLs for both upload and download. `content` plugin calls `media_service` for all file operations.
4. **certificates trigger**: Issued when `learning_progress` reports 100% course completion. PDF generation via `printpdf` or similar Rust library. Stored as a file in `media_service`.
5. **search**: Postgres `tsvector` full-text search on `courses.title`, `courses.description`, `content_items.title`, `content_items.body`. No external search engine (Elastic/Meilisearch) in v1.
6. **Stub cleanup**: Delete `courses`, `files`, `discussions` plugin directories. Their functionality is covered by `course_management`, `content`+`media_service`, and `messaging` respectively.
7. **support scope v1**: Two surfaces in ONE `com.elupane.support` plugin — (a) help-desk **tickets** (open → staff reply/assign → close/resolve, reopenable) and (b) a curated **FAQ/knowledge-base** article list (viewable by all; authored by admins/support).
8. **support routing/notifications**: Reuse the auth JWT + the shared `courses`/`users` DB. Emit events on new ticket/reply/article so the existing `notifications` plugin alerts staff (new tickets), requesters (replies), and admins (article changes).

---

## Task 1: Wire up `messaging` plugin

### 1a. Add `handlers.rs`

Create `plugins/messaging/src/handlers.rs` following the exact pattern from `plugins/enrollments/src/handlers.rs`:

```rust
use crate::dto::*;
use crate::services;
use crate::PLUGIN_ID;
use elupane_core::PluginMessage;

pub async fn dispatch(pool: &sqlx::PgPool, command: PluginMessage) -> anyhow::Result<elupane_core::PluginMessage> {
    let op = &command.op;
    let payload = &command.payload;

    match op.as_str() {
        "messaging.discussion.list" => { ... }
        "messaging.discussion.create" => { ... }
        "messaging.discussion.reply" => { ... }
        "messaging.discussion.list_replies" => { ... }
        "messaging.reaction.toggle" => { ... }
        _ => Err(anyhow::anyhow!("unknown operation: {}", op)),
    }
}
```

Operations to expose:
- `messaging.discussion.list` — list topics for a course
- `messaging.discussion.create` — create topic
- `messaging.discussion.reply` — add reply
- `messaging.discussion.list_replies` — paginated replies
- `messaging.reaction.toggle` — toggle emoji reaction on reply

### 1b. Update `main.rs`

Replace the heartbeat loop with the standard plugin bootstrap:

```rust
// 1. Redis pool
// 2. Postgres pool + run migrations
// 3. Announce capabilities: messaging.discussion.*, messaging.reaction.*
// 4. Enter dispatch loop: consume from elupane:commands:<plugin_id>
```

Capabilities to announce:
```
messaging.discussion.list
messaging.discussion.create
messaging.discussion.reply
messaging.discussion.list_replies
messaging.reaction.toggle
```

### 1c. Frontend wiring

Add `messaging_bridge.rs` in `plugins/frontend/src/`:
- `list_discussions(course_id)`
- `create_discussion(course_id, title, body)`
- `list_replies(discussion_id)`
- `create_reply(discussion_id, body)`
- `toggle_reaction(reply_id, emoji)`

Add `messaging_ui.rs` with Dioxus components for discussion thread view.

Add routes in `server_functions.rs` or Axum handlers in `main.rs`.

---

## Task 2: Implement `media_service`

### 2a. Create `services/media_service`

Create `D:\Work\E-Lupane\services\media_service\` as a standalone Rust binary/service (or integrate as a shared library crate used by plugins). Given the plugin architecture, make it a **plugin**:

```
plugins/media_service/
  Cargo.toml
  src/main.rs
  src/handlers.rs
  src/dto.rs
  src/models.rs
  src/repository.rs
  migrations/0001_media.sql
```

### 2b. Database schema

```sql
CREATE TABLE IF NOT EXISTS media_assets (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bucket      TEXT NOT NULL,
    object_key  TEXT NOT NULL,
    filename    TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes  BIGINT NOT NULL,
    uploaded_by UUID NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS media_assets_bucket_key_idx ON media_assets (bucket, object_key);
```

### 2c. Operations

- `media.upload.presign` — returns S3 presigned PUT URL + object key
- `media.download.presign` — returns S3 presigned GET URL
- `media.asset.info` — metadata for an asset
- `media.asset.delete` — soft delete

### 2d. S3 config

Read from `.env`:
- `S3_ENDPOINT` (or `MINIO_ENDPOINT`)
- `S3_BUCKET`
- `S3_ACCESS_KEY`
- `S3_SECRET_KEY`
- `S3_REGION`
- `S3_USE_PATH_STYLE` (for MinIO)

Use `aws-sdk-s3` or `rust-s3` crate.

### 2e. Integration

- `content` plugin calls `media.upload.presign` before allowing a file upload
- `assignments` plugin calls `media.upload.presign` for submission attachments
- Frontend uploads directly to S3 using the presigned URL, then confirms with the plugin

---

## Task 3: Build `certificates` plugin

### 3a. Create `plugins/certificates/`

```
plugins/certificates/
  Cargo.toml
  src/main.rs
  src/handlers.rs
  src/dto.rs
  src/models.rs
  src/repository.rs
  migrations/0001_certificates.sql
```

### 3b. Database schema

```sql
CREATE TABLE IF NOT EXISTS certificates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    course_id       UUID NOT NULL,
    certificate_no  TEXT NOT NULL UNIQUE,
    issued_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMPTZ,
    revoked_by      UUID,
    pdf_object_key  TEXT,
    verification_url TEXT NOT NULL
);
```

### 3c. Trigger

Subscribe to `learning_progress` events via the event bus. When `progress.course_completed` fires:
1. Generate certificate number (e.g., `ELUP-{year}-{uuid}`)
2. Generate PDF (use `printpdf` or `wkhtmltopdf` via CLI)
3. Upload PDF to `media_service`
4. Issue certificate row

### 3d. Operations

- `certificate.issue` — manual re-issue
- `certificate.revoke` — soft revoke
- `certificate.verify` — public verification by certificate number

### 3e. Frontend

- `certificate_bridge.rs` — RPC calls
- `certificate_ui.rs` — view/download certificate component
- Add "Certificate" tab to student dashboard when course is complete

---

## Task 4: Stand up `search`

### 4a. Extend `plugins/content/src/repository.rs`

Add a `search_content` function using Postgres full-text search:

```sql
SELECT id, course_id, module_id, title, body, kind, position,
       ts_rank(to_tsvector('english', coalesce(title,'') || ' ' || coalesce(body,'')), webquery) AS rank
FROM content_items
WHERE to_tsvector('english', coalesce(title,'') || ' ' || coalesce(body,'')) @@ webquery
  AND course_id = $2
ORDER BY rank DESC
LIMIT $3 OFFSET $4;
```

### 4b. Add `plugins/course_management/src/repository.rs` search

Search `courses` table by title + description + tags.

### 4c. Add `content.search` and `courses.search` operations

Wire through existing plugin handlers (no new plugin needed — extend existing ones).

### 4d. Frontend

- `search_bridge.rs` — `search_courses(query)` and `search_content(course_id, query)`
- `search_ui.rs` — search bar + results page
- Add search bar to header/nav

---

## Task 5: Clean up duplicate stubs

Delete these directories entirely:
- `plugins/courses/` — covered by `course_management`
- `plugins/files/` — covered by `content` + `media_service`
- `plugins/discussions/` — covered by `messaging`

Remove any references in:
- `AGANTS.md`
- `.kilo/plans/` (update if these are referenced)
- Root `Cargo.toml` workspace members (if present)

## Task 6: Build the `support` plugin (help-desk tickets + FAQ)

A user-support module delivered like every other module — a Rust plugin on the
Redis RPC backbone. One plugin (`com.elupane.support`) hosts two surfaces:
a help-desk ticket queue and a small FAQ/knowledge base.

### 6a. Schema — `plugins/support/migrations/0006_support.sql`

Idempotent raw SQL (IF NOT EXISTS / guarded), matching the other plugins' style in
the shared `elupane` database:

- `support_tickets(id, subject, category, priority, status, requester_id, assignee_id, course_id?, created_at, updated_at, closed_at)`
  - `status` in `open | in_progress | resolved | closed`, `priority` in `low | normal | high | urgent`
  - `course_id` optional (link a ticket to a course via the enrollments-style owner check)
- `support_messages(id, ticket_id, author_id, body, created_at)` — the ticket thread
- `support_faq(id, title, body, category, created_by, published, sort_order, created_at, updated_at)` — knowledge base

### 6b. Plugin

`plugins/support/` with `main.rs` (standard PluginClient bootstrap + capability
broadcast), `handlers.rs`, `services.rs`, `repository.rs`, `dto.rs`, `authz.rs`
(mirror `plugins/enrollments`). Announce `support.*` capabilities.

Operations:
- Tickets: `support.ticket.create`, `support.ticket.list`, `support.ticket.get`,
  `support.ticket.assign`, `support.ticket.set_status`
- Thread: `support.message.create` (reply), `support.message.list`
- FAQ: `support.faq.list`, `support.faq.get`, `support.faq.create`,
  `support.faq.update`, `support.faq.delete` (author only)

### 6c. Authz

Every call carries the auth-plugin JWT (`token`). Rules:
- `admin` / support role → any ticket, plus FAQ authoring (create/update/delete).
- Ticket creator → their own tickets and replies.
- Anyone authenticated → read + create tickets, read FAQ.
- Public FAQ read is optional in v1; default requires a valid session.

### 6d. Notifications

Publish `support.ticket.created`, `support.message.created`, and `support.faq.*`
events on the event bus so the `notifications` plugin alerts staff (new tickets),
the requester (replies), and admins (article changes).

### 6e. Frontend

- `support_bridge.rs` — RPC client to `com.elupane.support` (mirrors the other bridges).
- `support_ui.rs` — "Raise a ticket" form, "My tickets" list + thread view,
  FAQ browse/search, and a staff inbox (assign, set status, close; author FAQ).
- Link into the dashboard nav / sidebar.

### 6f. Workspace registration

- Add `plugins/support` to the root `Cargo.toml` workspace `members`.
- Add `plugins/support/plugin.toml` with `id = "com.elupane.support"`,
  `dependencies = ["com.elupane.auth", "com.elupane.notifications"]`.

---

## Implementation Order

| Step | Task | Estimated Complexity |
|------|------|---------------------|
| 1 | Wire `messaging` handlers + main.rs | Medium |
| 2 | Frontend messaging UI + bridge | Medium |
| 3 | Create `media_service` plugin | Medium |
| 4 | Integrate `content` + `assignments` with `media_service` | Medium |
| 5 | Create `certificates` plugin | Medium |
| 6 | Wire `learning_progress` → `certificates` event | Low |
| 7 | Add Postgres `tsvector` search to `content` + `courses` | Low |
| 8 | Frontend search UI | Low |
| 9 | Delete duplicate stubs | Low |
| 10 | `support` plugin backend (schema + handlers) | Medium |
| 11 | `support` FAQ + frontend UI + notifications wiring | Medium |

Total: ~6-8 focused implementation tasks.

---

## Validation Plan

For each task:
1. `cargo build --workspace` → 0 errors
2. `cargo test -p <plugin>` → tests pass
3. Manual smoke test via frontend UI

Specific validations:
- **messaging**: Create discussion topic, add reply, toggle reaction — all visible in UI
- **media_service**: Upload file via presigned URL, download it, verify content-type
- **certificates**: Complete a course → PDF generated → download works
- **search**: Search for "intro" → returns matching courses/content
- **stubs**: `cargo build --workspace` still succeeds after deletions
- **support**: A student opens a ticket → staff replies → the student replies → staff resolves/closes it. The student list shows only their own tickets; staff sees all. An admin authors + publishes an FAQ article that a signed-in user can view. Each action fires a `notifications` event.

---

## Risks

1. **media_service S3 compatibility**: If the deployment uses MinIO, ensure path-style URLs work. Test both MinIO and real S3.
2. **messaging scope creep**: Direct messaging is modeled but not wired — keep it out of v1.
3. **Search performance**: `tsvector` is fine for small catalogs. If content grows beyond ~100k rows, add `GIN` indexes.
4. **Certificate PDF generation**: `printpdf` is low-level; consider `wkhtmltopdf` binary dependency if PDFs need complex layouts.
5. **support**: Who counts as "support staff" (a new role vs. reuse `admin`)? Do tickets need course/enrollment context? Rate-limit ticket creation; in-app only vs. email notifications; FAQ categories vs. free-text search in v1.

---

## Open Questions

1. **S3 provider**: Should we support AWS S3, MinIO, and GCS via a trait, or lock to S3-compatible only? **Recommendation**: S3-compatible only (covers MinIO, AWS, Wasabi, Cloudflare R2).
2. **Certificate template**: Should the PDF layout be configurable (HTML template) or hardcoded Rust code? **Recommendation**: HTML template rendered by `wkhtmltopdf` for flexibility.
3. **Search scope v1**: Search only `courses` + `content_items`, or also `assignments` + `quizzes`? **Recommendation**: courses + content only in v1; extend later.
4. **messaging notifications**: Should new replies trigger `notifications` plugin events? **Recommendation**: Yes, but wire in v2 after messaging core is stable.
5. **support role model**: Introduce a dedicated `support`/`helpdesk` role, or reuse `admin`? **Recommendation**: a `support` role granted the `support.*` permissions, seeded via the auth plugin, with `admin` as a fallback.
6. **support email**: Should ticket activity also go out by email, or in-app only via `notifications`? **Recommendation**: in-app only in v1; email later.
