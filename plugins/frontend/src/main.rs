#![allow(non_snake_case)]

use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

// `db` holds the SQLx (PostgreSQL) connection pool. It is server-only because
// SQLx and the PostgreSQL driver cannot be compiled to WASM.
#[cfg(feature = "server")]
mod db;

// Dioxus "server functions". This module is compiled on BOTH the client and the
// server: on the client it generates a stub that calls the server over HTTP,
// while on the server the function bodies run (and may use SQLx via `crate::db`).
mod server_functions;

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
    let mut count = use_signal(|| 0);
    // `use_action` calls a Dioxus server function (an Axum endpoint) from the
    // client and tracks its result/loading/error state.
    let mut health = use_action(|| async move { server_functions::health().await });
    let mut courses = use_action(|| async move { server_functions::list_courses().await });

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div {
            class: "app",
            h1 { "E-Lupane · Dioxus 0.7 Fullstack 👋" }
            p { "Dioxus + SQLx + Axum — served on port 8090." }

            button {
                class: "btn",
                onclick: move |_| count += 1,
                "Client counter: {count}"
            }

            button {
                class: "btn",
                onclick: move |_| health.call(),
                "Check /api/health (SQLx + Axum)"
            }
            p { "Health: {health.value():?}" }

            button {
                class: "btn",
                onclick: move |_| courses.call(),
                "List courses (SQLx + Axum)"
            }
            p { "Courses: {courses.value():?}" }
        }
    }
}
