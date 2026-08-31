//! Assignments plugin: assignment lifecycle, submissions with deadline/late
//! enforcement, resources, grading, rubrics.
//!
//! Startup: Redis → PostgreSQL (migrations) → announce capabilities →
//! consume commands from `elupane:commands:com.elupane.assignments` → reply → ack.

mod authz;
mod dto;
mod handlers;
mod models;
mod repository;
mod services;
mod storage;

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tokio::signal;

use rust_sdk::PluginClient;

const PLUGIN_ID: &str = handlers::PLUGIN_ID;
const PLUGIN_VERSION: &str = "1.0.0";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    println!("Assignments Plugin Started");

    // 1. Redis — the inter-plugin messaging backbone.
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    cache::connection::connect(&redis_url).await?;
    println!("[assignments] connected to Redis at {redis_url}");

    // 2. PostgreSQL.
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://elupane:elupane@localhost:5432/elupane".into());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    // The auth plugin already owns migration `0001` in the shared
    // `_sqlx_migrations` table, so `sqlx::migrate!` cannot be used here (it
    // would report version 1 as "previously applied but missing"). Instead we
    // run our schema directly — every statement is idempotent (`IF NOT EXISTS`).
    sqlx::raw_sql(include_str!("../migrations/0002_assignments.sql"))
        .execute(&pool)
        .await?;
    println!("[assignments] database ready");

    // 3. Messaging client + capability broadcast.
    let mut client = PluginClient::connect(PLUGIN_ID).await?;

    let mut capabilities = permissions::CapabilitySet::new();
    for capability in [
        "assignment.create",
        "assignment.read",
        "assignment.update",
        "assignment.delete",
        "assignment.resource.attach",
        "assignment.resource.remove",
        "assignment.resource.download",
        "submission.submit",
        "submission.status",
        "submission.download",
        "grade.create",
        "grade.get",
        "grade.history",
        "rubric.create",
        "rubric.get",
        "rubric.list",
    ] {
        capabilities.grant(capability);
    }
    let announcer = client.announce(PLUGIN_VERSION, capabilities.to_vec()).await?;
    println!("[assignments] capabilities announced");

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
                    println!("[assignments] heartbeat");
                }
            }
            command = client.next_command() => {
                match command {
                    Ok(consumed) => {
                        println!("[assignments] executing {}", consumed.message.op);
                        let reply = handlers::handle_command(&pool, &consumed.message).await;
                        if let Err(err) = client.send_reply(&reply).await {
                            println!("[assignments] failed to send reply: {err}");
                        }
                        if let Err(err) = client.ack(&consumed).await {
                            println!("[assignments] failed to ack: {err}");
                        }
                    }
                    Err(err) => {
                        println!("[assignments] consume error: {err}");
                    }
                }
            }
            _ = signal::ctrl_c() => {
                println!("[assignments] shutting down");
                break;
            }
        }
    }

    Ok(())
}
