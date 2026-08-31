//! Idempotency guard for at-least-once command redelivery.
//!
//! Redis Streams deliver each message at least once (a crashed consumer leaves
//! it pending). Handlers should guard with an idempotency key so a redelivery
//! does not execute the operation twice.

use anyhow::Result;
use redis::AsyncCommands;

pub struct IdempotencyGuard;

impl IdempotencyGuard {
    /// Claim `key` for `window_secs`.
    ///
    /// Returns `true` if the key was **already claimed** within the window
    /// (i.e. this message has been seen before and should be skipped).
    pub async fn already_processed(key: &str, window_secs: u64) -> Result<bool> {
        let mut con = cache::connection::conn().await?;
        let claimed: bool = con.set_nx(key, "1").await?;
        if claimed {
            let _: () = con.expire(key, window_secs as i64).await?;
            Ok(false)
        } else {
            Ok(true)
        }
    }
}
