# Google OAuth Authentication with Local Permission Matrix

## Goal

Replace the custom username/password login (auth plugin over Redis RPC) with Google OAuth. The user's `name` and `surname` come from the Google account profile. Local role assignment is derived from `.env` allow-lists; any signed-in Google account not explicitly listed is treated as a `student`.

The auth plugin (PostgreSQL-backed `bcrypt` + JWT) stays in place for **backend** service-to-service auth (e.g. content/assignments/courses RPC between the frontend server and the auth plugin), but the **frontend user-facing login is Google-only**.

## Current State (from code review)

- `plugins/auth/` is a fully working plugin (bcrypt + JWT, roles in Postgres). `plugins/frontend/src/auth_bridge.rs` wraps it over Redis RPC.
- `plugins/frontend/src/server_functions.rs:55-70` already references `crate::google_classroom_bridge::verify_session(&token)`, but **no such module exists** — that endpoint currently fails to compile/link.
- `.env` (root) already declares `GOOGLE_OAUTH_CLIENT_ID`, `GOOGLE_OAUTH_CLIENT_SECRET`, `GOOGLE_OAUTH_REDIRECT_URI`, plus four allow-lists:
  - `GOOGLE_CLASSROOM_SUPER_ADMINS`
  - `GOOGLE_CLASSROOM_ADMINS`
  - `GOOGLE_CLASSROOM_DEVELOPERS`
  - `GOOGLE_CLASSROOM_LECTURERS`
  - (no `STUDENTS` list — students are the default)
- Frontend currently has a username/password login form. The session token is stored in `localStorage`; `server_functions::auth_login` + `auth_verify` (via auth_bridge) are its only call paths.
- The Cargo.toml for `plugins/frontend` does **not** currently depend on `oauth2`, `reqwest`, or `jsonwebtoken` (those would be needed for Google OAuth). `tokio` is already optional under `server`.

## Decisions Locked From User Statement

1. **Authentication source**: Google OAuth only (no passwords for end users).
2. **Identity source of truth**: `name` and `surname` are taken from the Google profile (`given_name` + `family_name`, with `name` split as fallback when only one is provided).
3. **Role assignment**: Read four allow-lists from `.env`. First match wins in priority order: `SUPER_ADMINS` → `ADMINS` → `DEVELOPERS` → `LECTURERS`. Anyone else = `student`.
4. **Permission matrix**: Lives **locally** in the frontend server's own tables (Postgres), not in the auth plugin. The auth plugin is no longer the user table; it's only an internal service-identity table for plugin RPC.

## Design

### 1. Login Flow (replaces password form)

1. User clicks **"Sign in with Google"** in the UI.
2. Browser hits a new server-fn `GET /api/auth/google/start` which 302-redirects to Google's authorization endpoint (scope `openid email profile`).
3. Google redirects back to `GOOGLE_OAUTH_REDIRECT_URI` (`/auth/google/callback`).
4. The callback route exchanges the `code` for tokens, calls Google's `userinfo` endpoint, and receives:
   - `email`, `email_verified`, `sub` (Google id), `given_name`, `family_name`, `name`, `picture`.
5. The server upserts a `local_user` row (see §2) and mints a **session token** (signed JWT or opaque random in Redis — recommended: opaque random stored in Redis with TTL, returned as cookie + returned in JSON for `localStorage`).
6. Browser receives the session and is redirected to `/` (app shell).

For dev / smoke tests, expose a second route `GET /api/auth/google/dev_login?email=...` that bypasses Google and synthesizes the same UserInfo shape (only when `GOOGLE_DEV_LOGIN=1` in `.env`).

### 2. Local Permission Matrix

A new table owned by the frontend crate (migrations live in `plugins/frontend/migrations/`, run by the existing installer):

```sql
CREATE TABLE local_user (
    id            UUID PRIMARY KEY,
    google_sub    TEXT UNIQUE NOT NULL,
    email         TEXT UNIQUE NOT NULL,
    given_name    TEXT NOT NULL,
    family_name   TEXT NOT NULL,
    full_name     TEXT NOT NULL,
    picture_url   TEXT,
    role          TEXT NOT NULL CHECK (role IN ('super_admin','admin','developer','lecturer','student')),
    active        BOOLEAN NOT NULL DEFAULT TRUE,
    last_login_at TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX local_user_email_idx ON local_user (email);
```

Sessions stored in Redis:

```
session:<token> -> JSON { user_id, role, email, exp }  (TTL 12h)
```

No password column. The local user table is a **projection of the Google identity** with the role assigned by the env-based matrix.

### 3. Role Resolution Algorithm

Read env vars at server start (cached in a `OnceLock<RoleMatrix>`):

```rust
fn resolve_role(email: &str) -> &'static str {
    let matrix = RoleMatrix::get();  // loaded from env once
    if matrix.super_admins.iter().any(|e| e.eq_ignore_ascii_case(email)) { return "super_admin"; }
    if matrix.admins       .iter().any(|e| e.eq_ignore_ascii_case(email)) { return "admin"; }
    if matrix.developers   .iter().any(|e| e.eq_ignore_ascii_case(email)) { return "developer"; }
    if matrix.lecturers    .iter().any(|e| e.eq_ignore_ascii_case(email)) { return "lecturer"; }
    "student"
}
```

The matrix is **not** read per-request — only on re-login. If you change `.env`, restart the server (or call a `reload` admin route — out of scope for v1).

### 4. Frontend Wiring (what changes for the user)

- Remove the `username`/`password` inputs and `auth_login` from the login page (`mock_pages.rs` `Login` page).
- Add a "Sign in with Google" button that navigates to `/api/auth/google/start`.
- `AuthUser` shape stays the same (`id`, `username`, `email`, `full_name`, `active`, `roles: Vec<String>`). `full_name` = `"<given_name> <family_name>"`. `username` = email local-part.
- `auth_verify` (already declared at `server_functions.rs:55-70`) is now the only path for session restore, and it goes through `google_classroom_bridge::verify_session`.
- The old `auth_login` (username/password) and `auth_roles` server functions are deleted.
- `auth_bridge.rs` (Redis RPC to the auth plugin) is left in place but no longer called from the login UI. It can be removed in a later cleanup.

### 5. Google Classroom Connection (already scaffolded)

`server_functions.rs:58` references `google_classroom_bridge::verify_session(token)`. The real module will:

```rust
// plugins/frontend/src/google_classroom_bridge.rs
pub struct GoogleUser { pub id: String, pub email: String, pub given_name: String,
    pub family_name: String, pub full_name: String, pub picture: Option<String> }

pub async fn exchange_code(code: &str) -> Result<GoogleUser>;
pub async fn verify_session(token: &str) -> Result<AuthUser>;  // -> 401 if no/invalid session
```

`verify_session` reads `session:<token>` from Redis, loads the `local_user` row, and returns the `AuthUser` projection used by the rest of the app. **This is the only function that needs to exist for the current `auth_verify` server function to compile** — the rest of this plan adds the start/callback/dev_login routes and the `local_user` table.

## Implementation Plan (ordered)

1. **Cargo deps** — `plugins/frontend/Cargo.toml`:
   - Add `oauth2 = "0.5"` (optional, `server` only)
   - Add `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"], optional }`
   - Add `serde_json` (already present)
   - Add `parking_lot` (optional) for the `RoleMatrix` `OnceLock` cache
   - Wire into the `server` feature.

2. **`plugins/frontend/src/google_classroom_bridge.rs`** (new, server-only via `#[cfg(feature = "server")]`):
   - `GoogleUser`, `AuthUser` (re-exported from `auth_bridge` for callers that still need it).
   - `RoleMatrix::from_env()`: parse the four `GOOGLE_CLASSROOM_*` lists (comma-separated, trim, lowercase).
   - `resolve_role(email)` per §3.
   - `pub async fn build_authorize_url(state: &str) -> Result<String>` — returns the Google OAuth URL.
   - `pub async fn exchange_code(code: &str) -> Result<GoogleUser>` — exchanges code, fetches `userinfo`, validates `email_verified`.
   - `pub async fn login_or_create(user: &GoogleUser) -> Result<(String, AuthUser)>` — upserts into `local_user` with role from matrix, creates a session in Redis, returns `(token, AuthUser)`.
   - `pub async fn verify_session(token: &str) -> Result<AuthUser>` — looks up session in Redis, loads `local_user` row.

3. **Migrations** — `plugins/frontend/migrations/0001_local_user.sql` with the `local_user` table from §2. Register with the existing installer migration runner (follow the pattern in `plugins/auth/migrations/`).

4. **DB module** — extend `plugins/frontend/src/db.rs` with a `local_user` repository: `upsert_from_google`, `find_by_id`, `set_last_login`. Reuse the existing `PgPool`.

5. **Sessions** — use `cache::connection` (already in deps) to `SET session:<token> <json> EX 43200`. Add a `google_sessions.rs` helper (or fold into the bridge).

6. **Server functions** — `plugins/frontend/src/server_functions.rs`:
   - Keep `auth_verify` exactly as it is at lines 55-70 (it will now compile).
   - Add `auth_google_start` (302 to Google) and `auth_google_callback` (Axum route — these are not pure server-fn POSTs; add them as plain Axum handlers in `main.rs`'s `serve` block alongside the Dioxus router).
   - Delete `auth_login` (lines 156-163) and the `ACCOUNT_TYPES` constant export usage from the login page.
   - Keep `users_list`, `users_create`, `users_delete`, `users_bulk_create` (admin UI uses them) but route them through the new `local_user` table, not the auth plugin. **Migration path**: add `local_user_admin_*` server functions and have the admin UI call those; deprecate the old ones next iteration.

7. **Login UI** — `mock_pages.rs` `Login` page:
   - Replace the username/password form with a single "Sign in with Google" button.
   - On click: `window.location.href = "/api/auth/google/start"`.
   - Surface error messages from `?error=...` query params on the callback redirect.
   - After successful login, redirect to `/dashboard`.

8. **`.env.example`** — document the four allow-list variables in the root `.env` (and add to `plugins/frontend/.env.example` if one exists). Add `GOOGLE_DEV_LOGIN=0` for the dev bypass.

9. **Dev bypass route** — `GET /api/auth/google/dev_login?email=...` (only when `GOOGLE_DEV_LOGIN=1`):
   - Skips Google, synthesizes `GoogleUser { email, given_name, family_name, full_name, .. }`.
   - Calls the same `login_or_create` to keep the rest of the flow identical.
   - Logs a `[auth] DEV login for <email>` warning.

10. **Logout** — `POST /api/auth/logout` deletes `session:<token>` from Redis and clears `localStorage` on the client.

11. **Tests** — in `plugins/frontend/src/google_classroom_bridge.rs`:
    - `role_matrix_prioritizes_super_admin_over_admin`
    - `role_matrix_defaults_to_student`
    - `resolve_role_is_case_insensitive`
    - `session_round_trip` (requires Redis; gate with `#[ignore]` if not available)

12. **Build verification** — `cargo check -p frontend --features server` must end with 0 errors. Run the same as the web target too.

## Affected Files

- `plugins/frontend/Cargo.toml` — new optional deps
- `plugins/frontend/migrations/0001_local_user.sql` — new
- `plugins/frontend/src/google_classroom_bridge.rs` — new (the only `google_*` module the codebase already references)
- `plugins/frontend/src/google_sessions.rs` — new (Redis session helpers)
- `plugins/frontend/src/db.rs` — add `local_user` repo functions
- `plugins/frontend/src/main.rs` — wire Axum routes (`/api/auth/google/*`, `/api/auth/logout`)
- `plugins/frontend/src/server_functions.rs` — delete `auth_login`, keep `auth_verify`, add admin functions for `local_user`
- `plugins/frontend/src/mock_pages.rs` — replace login form
- `plugins/frontend/src/auth_bridge.rs` — leave intact (still used for admin RPC to auth plugin), or delete in cleanup PR
- `.env` — document new vars (already present, no change needed unless adding `GOOGLE_DEV_LOGIN`)

## Open Questions

- **Session storage**: JWT cookie (stateless, harder to revoke) vs opaque token in Redis (recommended, used in the plan). Confirm.
- **Student allow-list**: not requested — any non-listed email is a student. Confirm this is intended (vs. blocking unknown emails entirely).
- **Auth plugin fate**: keep for plugin-to-plugin RPC, or fully deprecate? Plan currently keeps it untouched.
- **Frontend server admin functions for `local_user`**: in scope now (so admin UI still works) or deferred?

## Validation

- `cargo check -p frontend --features server` → 0 errors.
- `cargo check -p frontend --features web` → 0 errors.
- Manual: hit `GET /api/auth/google/dev_login?email=foo@bar` with `GOOGLE_DEV_LOGIN=1` → returns 302 to `/dashboard?token=...`.
- Manual: hit `POST /api/auth/verify { token }` → returns `AuthUser` with `full_name="Foo Bar"`, `roles=["student"]` for an unknown email.
- Manual: set `GOOGLE_CLASSROOM_ADMINS=me@x.com`, dev-login as that email → `roles=["admin"]`.
