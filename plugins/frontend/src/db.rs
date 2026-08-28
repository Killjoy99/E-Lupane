//! Server-only SQLx (PostgreSQL) connection pool.
//!
//! This module is only compiled when the `server` feature is enabled, because
//! SQLx and the PostgreSQL driver cannot be compiled to WASM (the client).

use std::sync::OnceLock;

use sqlx::{postgres::PgPoolOptions, PgPool};

static POOL: OnceLock<PgPool> = OnceLock::new();

/// Initialise the global connection pool (best-effort).
///
/// If PostgreSQL is not reachable we log a warning and continue, so the Dioxus
/// app still serves (e.g. `dx serve --port 8090`) and `/api/health` reports the
/// database as "unavailable" instead of crashing the whole server.
pub async fn init() {
    dotenvy::dotenv().ok();

    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/elupane?sslmode=disable".to_string()
    });

    match PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await
    {
        Ok(pool) => {
            let _ = POOL.set(pool);
            println!("[sqlx] connected to PostgreSQL at {url}");
        }
        Err(err) => {
            eprintln!("[sqlx] warning: could not connect to PostgreSQL at {url}: {err}");
        }
    }
}

/// The global pool, if it was successfully initialised.
pub fn pool() -> Option<&'static PgPool> {
    POOL.get()
}

/// Run `SELECT 1` to check PostgreSQL connectivity through the pool.
pub async fn ping() -> bool {
    match pool() {
        Some(pool) => sqlx::query("SELECT 1").fetch_one(pool).await.is_ok(),
        None => false,
    }
}
