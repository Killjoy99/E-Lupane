//! Auth plugin: user management + RBAC, driven entirely by Redis messages.
//!
//! Startup: Redis → PostgreSQL (migrations) → announce capabilities →
//! consume commands from `elupane:commands:com.elupane.auth` → reply → ack.

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
    println!("Auth Plugin Started");

    // 1. Redis — the inter-plugin messaging backbone.
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    cache::connection::connect(&redis_url).await?;
    println!("[auth] connected to Redis at {redis_url}");

    // 2. PostgreSQL — users, roles, permissions.
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://elupane:elupane@localhost:5432/elupane".into());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("[auth] database ready");

    // 3. First-run convenience: seed the default platform admin and the
    // demo account matrix required for development/test use.
    ensure_default_admin(&pool).await?;
    services::ensure_seed_accounts(&pool).await?;

    // 4. Messaging client + capability broadcast.
    let mut client = PluginClient::connect(PLUGIN_ID).await?;

    let mut capabilities = permissions::CapabilitySet::new();
    for capability in [
        "auth.user.create",
        "auth.user.read",
        "auth.user.update",
        "auth.user.delete",
        "auth.role.list",
        "auth.role.assign",
        "auth.permission.list",
        "auth.login",
        "auth.verify",
        "auth.google_login",
    ] {
        capabilities.grant(capability);
    }
    let announcer = client.announce(PLUGIN_VERSION, capabilities.to_vec()).await?;
    println!("[auth] capabilities announced");

    // 5. Reclaim this consumer's pending work from a previous run.
    for consumed in client.drain_pending().await? {
        let reply = handlers::handle_command(&pool, &consumed.message).await;
        let _ = client.send_reply(&reply).await;
        let _ = client.ack(&consumed).await;
    }

    // 6. Main loop: heartbeat + consume + reply + ack.
    let mut heartbeat = tokio::time::interval(Duration::from_secs(10));
    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if announcer.heartbeat().await.is_ok() {
                    println!("[auth] heartbeat");
                }
            }
            command = client.next_command() => {
                match command {
                    Ok(consumed) => {
                        println!("[auth] executing {}", consumed.message.op);
                        let reply = handlers::handle_command(&pool, &consumed.message).await;
                        if let Err(err) = client.send_reply(&reply).await {
                            println!("[auth] failed to send reply: {err}");
                        }
                        if let Err(err) = client.ack(&consumed).await {
                            println!("[auth] failed to ack: {err}");
                        }
                    }
                    Err(err) => {
                        println!("[auth] consume error: {err}");
                    }
                }
            }
            _ = signal::ctrl_c() => {
                println!("[auth] shutting down");
                break;
            }
        }
    }

    Ok(())
}

/// Seed `admin / admin123` (overridable via env) when the users table is empty.
async fn ensure_default_admin(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    if repository::count_users(pool).await? > 0 {
        return Ok(());
    }

    let req = crate::dto::CreateUserRequest {
        username: std::env::var("AUTH_ADMIN_USERNAME").unwrap_or_else(|_| "admin".into()),
        email: std::env::var("AUTH_ADMIN_EMAIL").unwrap_or_else(|_| "admin@elupane.local".into()),
        password: std::env::var("AUTH_ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".into()),
        full_name: "Platform Administrator".into(),
        roles: vec!["admin".into()],
        admin_token: None,
    };

    let view = services::create_user(pool, req).await?;
    println!("[auth] created default admin '{}'", view.username);
    Ok(())
}
