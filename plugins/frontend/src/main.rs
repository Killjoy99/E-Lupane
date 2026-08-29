#![allow(non_snake_case)]

use dioxus::prelude::*;

use dashboard::Dashboard;

const MAIN_CSS: Asset = asset!("/assets/main.css");

// `db` holds the SQLx (PostgreSQL) connection pool. It is server-only because
// SQLx and the PostgreSQL driver cannot be compiled to WASM.
#[cfg(feature = "server")]
mod db;

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
        use dioxus::server::axum::routing::get;
        use dioxus::server::router;

        // Initialise the SQLx pool (best-effort, so the app still serves even if
        // PostgreSQL is not reachable yet — `/api/health` will report it).
        db::init().await;

        Ok(router(app)
            // A plain Axum route, to show the server really is Axum.
            .route(
                "/api/about",
                get(|| async { "E-Lupane · Dioxus + SQLx + Axum (port 8090)" }),
            ))
    });

    // ---- Client / other platforms ----
    #[cfg(not(feature = "server"))]
    dioxus::launch(app);
}

fn app() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Dashboard {}
    }
}
