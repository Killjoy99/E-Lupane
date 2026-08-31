//! Redis connection management.
//!
//! A single global `redis::Client` is opened once at boot. Connections are
//! multiplexed (a shared underlying socket per handle), so callers simply ask
//! for a fresh handle with [`conn`] and run their commands on it.

use std::sync::OnceLock;

use anyhow::Result;
use redis::aio::MultiplexedConnection;
use redis::Client;

static CLIENT: OnceLock<Client> = OnceLock::new();

/// Open (or reuse) the global Redis client and verify connectivity.
///
/// Fails fast if Redis is unreachable so misconfiguration is caught at boot.
/// Idempotent: a second call after a successful connect is a no-op.
pub async fn connect(redis_url: &str) -> Result<()> {
    if CLIENT.get().is_some() {
        return Ok(());
    }

    let client = Client::open(redis_url)?;

    let mut con = client.get_multiplexed_async_connection().await?;
    let _pong: String = redis::cmd("PING").query_async(&mut con).await?;

    CLIENT
        .set(client)
        .map_err(|_| anyhow::anyhow!("Redis client already initialized"))?;

    Ok(())
}

/// The global Redis client (panics if [`connect`] was never called).
pub fn client() -> &'static Client {
    CLIENT.get().expect("Redis client not initialized")
}

/// Open a multiplexed connection for issuing commands.
pub async fn conn() -> Result<MultiplexedConnection> {
    Ok(client().get_multiplexed_async_connection().await?)
}

/// True once [`connect`] has been called successfully.
pub fn is_initialized() -> bool {
    CLIENT.get().is_some()
}

