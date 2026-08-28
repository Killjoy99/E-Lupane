//! Dioxus "server functions".
//!
//! Each server function is an **Axum** endpoint on the fullstack server. This
//! module is compiled on both the client and the server:
//!
//! * on the **client**, `#[get]` generates a stub that calls the endpoint over HTTP;
//! * on the **server**, the function bodies run and may use SQLx through
//!   [`crate::db`] (SQLx is server-only).
//!
//! See the official docs: https://dioxuslabs.com/learn/0.7/server/fullstack/

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// `/api/health` — check that the server is up and that PostgreSQL is reachable
/// via the SQLx pool.
#[get("/api/health")]
pub async fn health() -> anyhow::Result<Health> {
    let database = if crate::db::ping().await {
        "connected".to_string()
    } else {
        "unavailable".to_string()
    };

    Ok(Health {
        status: "ok".to_string(),
        database,
    })
}

/// `/api/courses` — read rows from the `courses` table using SQLx.
///
/// Requires the E-Lupane migrations to have created the `courses` table. If the
/// table does not exist yet (migrations are currently empty) this returns a
/// helpful error rather than failing the whole server.
#[get("/api/courses")]
pub async fn list_courses() -> anyhow::Result<Vec<Course>> {
    let pool =
        crate::db::pool().ok_or_else(|| anyhow::anyhow!("database not available"))?;

    // `sqlx::Row` gives us column access via `row.get("...")`.
    use sqlx::Row as _;

    let rows = sqlx::query(
        "SELECT id::bigint AS id, course_code, course_name, COALESCE(active, false) AS active \
         FROM courses ORDER BY course_name LIMIT 100",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|row| Course {
            id: row.get("id"),
            course_code: row.get("course_code"),
            course_name: row.get("course_name"),
            active: row.get("active"),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub status: String,
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: i64,
    pub course_code: String,
    pub course_name: String,
    pub active: bool,
}
