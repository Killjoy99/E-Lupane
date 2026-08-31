//! Redis connectivity for the E-Lupane platform.
//!
//! `connection` manages the shared client, `pubsub` wraps Pub/Sub for
//! broadcasts/events, and `session` provides an expiring session store.

pub mod connection;
pub mod pubsub;
pub mod session;

