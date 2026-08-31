//! Enrollments plugin: course enrollment lifecycle, rosters, invitations,
//! history and content-access enforcement.
//!
//! Startup: Redis → PostgreSQL (schema) → announce capabilities → consume
//! commands from `elupane:commands:com.elupane.enrollments` → reply → ack.

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
    println!("Enrollments Plugin Started");

    // 1. Redis — the inter-plugin messaging backbone.
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    cache::connection::connect(&redis_url).await?;
    println!("[enrollments] connected to Redis at {redis_url}");

    // 2. PostgreSQL. The auth plugin owns migration `0001`, assignments `0002`
    // and course_management `0003` in the shared `_sqlx_migrations` table, so
    // `sqlx::migrate!` cannot be used. Our schema is applied with idempotent
    // raw SQL (`IF NOT EXISTS`), so it is safe alongside the other plugins.
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://elupane:elupane@localhost:5432/elupane".into());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::raw_sql(include_str!("../migrations/0004_enrollments.sql"))
        .execute(&pool)
        .await?;
    println!("[enrollments] database ready");

    // 3. Messaging client + capability broadcast.
    let mut client = PluginClient::connect(PLUGIN_ID).await?;

    let mut capabilities = permissions::CapabilitySet::new();
    for capability in ["enrollments.read", "enrollments.write", "events.subscribe"] {
        capabilities.grant(capability);
    }
    let announcer = client.announce(PLUGIN_VERSION, capabilities.to_vec()).await?;
    println!("[enrollments] capabilities announced");

    // 4. Reclaim this consumer's pending work from a previous run.
    for consumed in client.drain_pending().await? {
        let reply = handlers::handle_command(&pool, &consumed.message).await;
        let _ = client.send_reply(&reply).await;
        let _ = client.ack(&consumed).await;
    }

    // 5. Main loop: heartbeat + expiry sweep + consume + reply + ack.
    let mut heartbeat = tokio::time::interval(Duration::from_secs(10));
    // Enrollment expiration: active enrollments past `expires_at` become
    // `expired` (with a history entry) every sweep.
    let mut sweep = tokio::time::interval(Duration::from_secs(60));
    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if announcer.heartbeat().await.is_ok() {
                    println!("[enrollments] heartbeat");
                }
            }
            _ = sweep.tick() => {
                match services::sweep_expired(&pool).await {
                    Ok(0) => {}
                    Ok(n) => println!("[enrollments] expired {n} enrollment(s)"),
                    Err(err) => println!("[enrollments] expiry sweep failed: {err}"),
                }
            }
            command = client.next_command() => {
                match command {
                    Ok(consumed) => {
                        println!("[enrollments] executing {}", consumed.message.op);
                        let reply = handlers::handle_command(&pool, &consumed.message).await;
                        if let Err(err) = client.send_reply(&reply).await {
                            println!("[enrollments] failed to send reply: {err}");
                        }
                        if let Err(err) = client.ack(&consumed).await {
                            println!("[enrollments] failed to ack: {err}");
                        }
                    }
                    Err(err) => {
                        println!("[enrollments] consume error: {err}");
                    }
                }
            }
            _ = signal::ctrl_c() => {
                println!("[enrollments] shutting down");
                break;
            }
        }
    }

    Ok(())
}
