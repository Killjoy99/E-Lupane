//! Academics plugin: faculties, departments, programs, academic years, semesters,
//! classes / cohorts, course offerings.
//!
//! Startup: Redis → PostgreSQL (schema) → announce capabilities → consume
//! commands from `elupane:commands:com.elupane.academics` → reply → ack.

mod authz;
mod dto;
mod handlers;
mod models;
mod repository;
mod services;

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tokio::signal;

use rust_sdk::PluginClient;

const PLUGIN_ID: &str = handlers::PLUGIN_ID;
const PLUGIN_VERSION: &str = "1.0.0";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    println!("Academics Plugin Started");

    // 1. Redis — the inter-plugin messaging backbone.
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    cache::connection::connect(&redis_url).await?;
    println!("[academics] connected to Redis at {redis_url}");

    // 2. PostgreSQL. The auth plugin owns `0001`, assignments `0002`,
    // course_management `0003`, enrollments `0004`, institution_management
    // `0005` in the shared `_sqlx_migrations` table, so `sqlx::migrate!` cannot
    // be used. Our schema is applied with idempotent raw SQL (`IF NOT EXISTS`
    // + guarded ALTER), so it is safe to run alongside the other plugins.
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://elupane:elupane@localhost:5432/elupane".into());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::raw_sql(include_str!("../migrations/0006_academics.sql"))
        .execute(&pool)
        .await?;
    println!("[academics] database ready");

    // 3. Messaging client + capability broadcast.
    let mut client = PluginClient::connect(PLUGIN_ID).await?;

    let mut capabilities = permissions::CapabilitySet::new();
    for capability in [
        "academics.faculty.read",
        "academics.faculty.write",
        "academics.department.read",
        "academics.department.write",
        "academics.program.read",
        "academics.program.write",
        "academics.year.read",
        "academics.year.write",
        "academics.semester.read",
        "academics.semester.write",
        "academics.class.read",
        "academics.class.write",
        "academics.offering.read",
        "academics.offering.write",
        "events.subscribe",
    ] {
        capabilities.grant(capability);
    }
    let announcer = client.announce(PLUGIN_VERSION, capabilities.to_vec()).await?;
    println!("[academics] capabilities announced");

    // 4. Reclaim this consumer's pending work from a previous run.
    for consumed in client.drain_pending().await? {
        let reply = handlers::handle_command(&pool, &consumed.message).await;
        let _ = client.send_reply(&reply).await;
        let _ = client.ack(&consumed).await;
    }

    // 5. Main loop: heartbeat + consume + reply + ack.
    let mut heartbeat = tokio::time::interval(Duration::from_secs(10));
    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if announcer.heartbeat().await.is_ok() {
                    println!("[academics] heartbeat");
                }
            }
            command = client.next_command() => {
                match command {
                    Ok(consumed) => {
                        println!("[academics] executing {}", consumed.message.op);
                        let reply = handlers::handle_command(&pool, &consumed.message).await;
                        if let Err(err) = client.send_reply(&reply).await {
                            println!("[academics] failed to send reply: {err}");
                        }
                        if let Err(err) = client.ack(&consumed).await {
                            println!("[academics] failed to ack: {err}");
                        }
                    }
                    Err(err) => {
                        println!("[academics] consume error: {err}");
                    }
                }
            }
            _ = signal::ctrl_c() => {
                println!("[academics] shutting down");
                break;
            }
        }
    }

    Ok(())
}
