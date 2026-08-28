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
