//! Notification system plugin: in-app, email, push notifications with preferences.
//!
//! Startup: Redis → PostgreSQL (schema) → announce capabilities → consume
//! commands from `elupane:commands:com.elupane.notifications` → reply → ack.

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

const PLUGIN_ID: &str = "com.elupane.notifications";
const PLUGIN_VERSION: &str = "1.0.0";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    println!("Notifications Plugin Started");

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    cache::connection::connect(&redis_url).await?;
    println!("[notifications] connected to Redis at {redis_url}");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://elupane:elupane@localhost:5432/elupane".into());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::raw_sql(include_str!("../migrations/0012_notifications.sql"))
        .execute(&pool)
        .await?;
    println!("[notifications] database ready");

    let mut client = PluginClient::connect(PLUGIN_ID).await?;

    let mut capabilities = permissions::CapabilitySet::new();
    for capability in [
        "notifications.read",
        "notifications.write",
        "notifications.preferences.read",
        "notifications.preferences.write",
        "events.subscribe",
    ] {
        capabilities.grant(capability);
    }
    let announcer = client.announce(PLUGIN_VERSION, capabilities.to_vec()).await?;
    println!("[notifications] capabilities announced");

    // Consume support-domain events and turn them into in-app notifications.
    {
        let pool = pool.clone();
        tokio::spawn(async move {
            use futures_util::StreamExt;
            let mut stream = match messaging_core::events::subscribe_events("support").await {
                Ok(stream) => Box::pin(stream),
                Err(err) => {
                    eprintln!("[notifications] support event subscribe failed: {err}");
                    return;
                }
            };
            while let Some(event) = stream.next().await {
                if let Err(err) = services::handle_support_event(&pool, &event).await {
                    eprintln!("[notifications] failed to ingest support event: {err}");
                }
            }
        });
    }

    for consumed in client.drain_pending().await? {
        let reply = handlers::handle_command(&pool, &consumed.message).await;
        let _ = client.send_reply(&reply).await;
        let _ = client.ack(&consumed).await;
    }

    let mut heartbeat = tokio::time::interval(Duration::from_secs(10));
    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if announcer.heartbeat().await.is_ok() {
                    println!("[notifications] heartbeat");
                }
            }
            command = client.next_command() => {
                match command {
                    Ok(consumed) => {
                        println!("[notifications] executing {}", consumed.message.op);
                        let reply = handlers::handle_command(&pool, &consumed.message).await;
                        if let Err(err) = client.send_reply(&reply).await {
                            println!("[notifications] failed to send reply: {err}");
                        }
                        if let Err(err) = client.ack(&consumed).await {
                            println!("[notifications] failed to ack: {err}");
                        }
                    }
                    Err(err) => {
                        println!("[notifications] consume error: {err}");
                    }
                }
            }
            _ = signal::ctrl_c() => {
                println!("[notifications] shutting down");
                break;
            }
        }
    }

    Ok(())
}
