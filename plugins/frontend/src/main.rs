#![allow(non_snake_case)]

use dioxus::prelude::*;

use dashboard::Dashboard;

const MAIN_CSS: Asset = asset!("/assets/main1.css");
const DASHBOARD_CSS: Asset = asset!("/assets/dashboard.css");

// `db` holds the SQLx (PostgreSQL) connection pool. It is server-only because
// SQLx and the PostgreSQL driver cannot be compiled to WASM.
#[cfg(feature = "server")]
mod db;

// Server-only bridge to the auth plugin over the Redis messaging layer.
#[cfg(feature = "server")]
mod auth_bridge;

// Server-only bridge to Google OAuth ("Sign in with Google"): authorization
// code flow, local user table, and Redis-backed opaque sessions.
#[cfg(feature = "server")]
mod google_classroom_bridge;

// Shared assignment view types (client + server).
mod assignments_types;

// Shared course view types (client + server).
mod course_types;

// Shared enrollment view types (client + server).
mod enrollment_types;

// Shared live-class view types (client + server).
mod live_types;

// Server-only bridge to the assignments plugin over the Redis messaging layer.
#[cfg(feature = "server")]
mod assignments_bridge;

// Server-only bridge to the course_management plugin over the Redis messaging layer.
#[cfg(feature = "server")]
mod course_bridge;

// Server-only bridge to the enrollments plugin over the Redis messaging layer.
#[cfg(feature = "server")]
mod enrollment_bridge;

// Server-only bridge to the live_classes plugin over the Redis messaging layer.
#[cfg(feature = "server")]
mod live_bridge;

// Server-only WebSocket relay for live-session video/chat/whiteboard signaling.
#[cfg(feature = "server")]
mod live_signaling;

// Server-only Model Context Protocol endpoint (Streamable HTTP) that exposes
// the notification operations to external AI clients as MCP tools.
#[cfg(feature = "server")]
mod mcp_server;

// Shared content view types (client + server).
mod content_types;

// Server-only bridge to the content plugin over the Redis messaging layer.
#[cfg(feature = "server")]
mod content_bridge;

// Shared messaging view types (client + server).
mod messaging_types;

// Server-only bridge to the messaging plugin over the Redis messaging layer.
#[cfg(feature = "server")]
mod messaging_bridge;

// Shared support view types (client + server).
mod support_types;

// Server-only bridge to the support plugin over the Redis messaging layer.
#[cfg(feature = "server")]
mod support_bridge;

// Shared support UI (client + server).
mod support_ui;

// Shared dashboard UI modules.
mod ui_helpers;
mod assignments_ui;
mod courses_ui;
mod content_ui;
mod enrollments_ui;
mod live_room;
mod live_classes_ui;
mod mock_data;
mod mock_pages;

// Live RBAC administration (users + roles × permissions matrix).
mod admin_pages;

// Dioxus "server functions". This module is compiled on BOTH the client and the
// server: on the client it generates a stub that calls the server over HTTP,
// while on the server the function bodies run (and may use SQLx via `crate::db`).
mod server_functions;

// Material Design 3 UI components + the role-based dashboard.
mod components;
mod dashboard;

fn main() {
    // Load .env early (before dioxus::serve reads PORT/IP from the environment).
    // When launched as a plugin the working directory is set to `frontend/`,
    // so this picks up frontend/.env with DATABASE_URL, PORT, and IP.
    #[cfg(feature = "server")]
    dotenvy::dotenv().ok();

    // ---- Server ----
    // ---- Server ----
    // `dioxus::serve` runs the Axum server. `dioxus::server::router(app)` returns
    // the Axum Router that serves the Dioxus app (SSR + static assets + server
    // functions). We augment it with our own Axum route and initialise SQLx here.
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        use dioxus::server::axum::routing::{get, post};
        use dioxus::server::router;

        // Initialise the SQLx pool (best-effort, so the app still serves even if
        // PostgreSQL is not reachable yet — `/api/health` will report it).
        db::init().await;

        // Connect to Redis — the inter-plugin messaging backbone. Best-effort:
        // the app still serves if Redis is down; login will surface the error.
        auth_bridge::init().await;

        Ok(router(app)
            // A plain Axum route, to show the server really is Axum.
            .route(
                "/api/about",
                get(|| async { "E-Lupane · Dioxus + SQLx + Axum (port 8090)" }),
            )
            // Google OAuth ("Sign in with Google") authorization-code flow.
            .route(
                "/api/auth/google/start",
                get(google_classroom_bridge::start_handler),
            )
            .route(
                "/auth/google/callback",
                get(google_classroom_bridge::callback_handler),
            )
            .route(
                "/api/auth/google/dev_login",
                get(google_classroom_bridge::dev_login_handler),
            )
            // Real-time signaling relay for webrtc-provider live sessions:
            // WebRTC offer/answer/ICE, chat and whiteboard strokes.
            .route("/ws/live/{session_id}", get(live_signaling::ws_handler))
            // Model Context Protocol (Streamable HTTP): E-Lupane's notification
            // operations as AI-callable tools, authenticated with the same
            // session JWT the browser uses.
            .route(
                mcp_server::MCP_PATH,
                post(mcp_server::mcp_post)
                    .get(mcp_server::mcp_get)
                    .delete(mcp_server::mcp_delete)
                    .options(mcp_server::mcp_options),
            ))
    });

    // ---- Client / other platforms ----
    #[cfg(not(feature = "server"))]
    dioxus::launch(app);
}

fn app() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: DASHBOARD_CSS }
        Dashboard {}
    }
}
