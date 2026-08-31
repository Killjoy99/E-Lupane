//! Redis-backed session/token store (used by the auth plugin).
//!
//! Sessions are simple expiring string keys — ideal for opaque bearer tokens,
//! refresh-token denylists, and ephemeral per-user state.

use anyhow::Result;
use redis::AsyncCommands;

use crate::connection::conn;

/// Store `value` under `key` for `ttl_secs` seconds (auto-expiring).
pub async fn set(key: &str, value: &str, ttl_secs: u64) -> Result<()> {
    let mut con = conn().await?;
    let _: () = con.set_ex(key, value, ttl_secs).await?;
    Ok(())
}

/// Fetch a value if it exists.
pub async fn get(key: &str) -> Result<Option<String>> {
    let mut con = conn().await?;
    Ok(con.get(key).await?)
}

/// Delete a key (idempotent).
pub async fn delete(key: &str) -> Result<()> {
    let mut con = conn().await?;
    let _: () = con.del(key).await?;
    Ok(())
}

