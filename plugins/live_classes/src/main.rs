//! Live classes plugin: schedule/join live learning sessions, attendance,
//! chat persistence and session resources. Real-time transport (video,
//! screen share, whiteboard) for the `webrtc` provider is relayed by the
//! frontend's WebSocket signaling endpoint (`/ws/live/{session_id}`); this
//! plugin only owns the durable state and authorization, so other providers
//! (Jitsi/Zoom/Teams) can be swapped in without touching course logic.
//!
//! Startup: Redis → PostgreSQL (schema) → announce capabilities → consume
//! commands from `elupane:commands:com.elupane.live_classes` → reply → ack.

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
    println!("Live Classes Plugin Started");

    // 1. Redis — the inter-plugin messaging backbone.
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    cache::connection::connect(&redis_url).await?;
    println!("[live_classes] connected to Redis at {redis_url}");

    // 2. PostgreSQL. Applied with idempotent raw SQL so it is safe alongside
    // the other plugins' migrations in the shared database.
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://elupane:elupane@localhost:5432/elupane".into());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::raw_sql(include_str!("../migrations/0007_live_classes.sql"))
        .execute(&pool)
        .await?;
    println!("[live_classes] database ready");

    // 3. Messaging client + capability broadcast.
    let mut client = PluginClient::connect(PLUGIN_ID).await?;

    let mut capabilities = permissions::CapabilitySet::new();
    for capability in ["live_classes.read", "live_classes.write", "events.subscribe"] {
        capabilities.grant(capability);
    }
    let announcer = client.announce(PLUGIN_VERSION, capabilities.to_vec()).await?;
    println!("[live_classes] capabilities announced");

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
                    println!("[live_classes] heartbeat");
                }
            }
            command = client.next_command() => {
                match command {
                    Ok(consumed) => {
                        println!("[live_classes] executing {}", consumed.message.op);
                        let reply = handlers::handle_command(&pool, &consumed.message).await;
                        if let Err(err) = client.send_reply(&reply).await {
                            println!("[live_classes] failed to send reply: {err}");
                        }
                        if let Err(err) = client.ack(&consumed).await {
                            println!("[live_classes] failed to ack: {err}");
                        }
                    }
                    Err(err) => {
                        println!("[live_classes] consume error: {err}");
                    }
                }
            }
            _ = signal::ctrl_c() => {
                println!("[live_classes] shutting down");
                break;
            }
        }
    }

    Ok(())
}
