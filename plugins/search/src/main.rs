//! Search plugin: full-text search for courses and content items.
//!
//! Startup: Redis → PostgreSQL (schema) → announce capabilities → consume
//! commands from `elupane:commands:com.elupane.search` → reply → ack.

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
    println!("Search Plugin Started");

    // 1. Redis — the inter-plugin messaging backbone.
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    cache::connection::connect(&redis_url).await?;
    println!("[search] connected to Redis at {redis_url}");

    // 2. PostgreSQL.
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://elupane:elupane@localhost:5432/elupane".into());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::raw_sql(include_str!("../migrations/0018_search.sql"))
        .execute(&pool)
        .await?;
    println!("[search] database ready");

    // 3. Messaging client + capability broadcast.
    let mut client = PluginClient::connect(PLUGIN_ID).await?;

    let mut capabilities = permissions::CapabilitySet::new();
    for capability in [
        "search.courses",
        "search.content",
        "search.all",
        "events.subscribe",
    ] {
        capabilities.grant(capability);
    }
    let announcer = client.announce(PLUGIN_VERSION, capabilities.to_vec()).await?;
    println!("[search] capabilities announced");

    // 4. Reclaim pending work.
    for consumed in client.drain_pending().await? {
        let reply = handlers::handle_command(&pool, &consumed.message).await;
        let _ = client.send_reply(&reply).await;
        let _ = client.ack(&consumed).await;
    }

    // 5. Main loop.
    let mut heartbeat = tokio::time::interval(Duration::from_secs(10));
    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if announcer.heartbeat().await.is_ok() {
                    println!("[search] heartbeat");
                }
            }
            command = client.next_command() => {
                match command {
                    Ok(consumed) => {
                        println!("[search] executing {}", consumed.message.op);
                        let reply = handlers::handle_command(&pool, &consumed.message).await;
                        if let Err(err) = client.send_reply(&reply).await {
                            println!("[search] failed to send reply: {err}");
                        }
                        if let Err(err) = client.ack(&consumed).await {
                            println!("[search] failed to ack: {err}");
                        }
                    }
                    Err(err) => {
                        println!("[search] consume error: {err}");
                    }
                }
            }
            _ = signal::ctrl_c() => {
                println!("[search] shutting down");
                break;
            }
        }
    }

    Ok(())
}
