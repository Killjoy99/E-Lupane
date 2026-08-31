//! Live integration tests for the Redis messaging layer.
//!
//! These require a running Redis on `127.0.0.1:6379`:
//!
//! ```sh
//! cargo test -p messaging_core --test messaging -- --ignored --nocapture
//! ```

use std::time::Duration;

use elupane_core::message::{keys, Message};
use messaging_core::{CapabilityAnnouncer, CommandConsumer, RequestClient};

#[tokio::test]
#[ignore = "requires a running Redis server"]
async fn capability_announce_and_registry() {
    cache::connection::connect("redis://127.0.0.1:6379")
        .await
        .unwrap();

    let announcer = CapabilityAnnouncer::new("com.smoke", "0.1.0", vec!["smoke.read".into()], 5);
    announcer.announce().await.unwrap();

    let listed = messaging_core::list_capabilities().await.unwrap();
    assert!(listed.iter().any(|a| a.plugin_id == "com.smoke"));
}

#[tokio::test]
#[ignore = "requires a running Redis server"]
async fn command_request_response_round_trip() {
    cache::connection::connect("redis://127.0.0.1:6379")
        .await
        .unwrap();

    let plugin_id = "com.smoke.consumer";
    let mut consumer = CommandConsumer::connect(plugin_id).await.unwrap();

    // Caller sends a command to the plugin's stream and awaits the reply.
    let caller = RequestClient::new("gateway").with_timeout(Duration::from_secs(5));
    let handle = tokio::spawn(async move {
        let reply = caller
            .request(plugin_id, "smoke.echo", serde_json::json!({ "value": 42 }))
            .await
            .unwrap();
        reply.payload["value"].as_i64().unwrap()
    });

    // The plugin consumes the command, publishes a reply, then acks.
    let consumed = consumer.next_message().await.unwrap();
    assert_eq!(consumed.message.op, "smoke.echo");

    let reply = Message::reply_to(&consumed.message, plugin_id, consumed.message.payload.clone());
    let channel = keys::reply_channel(reply.correlation_id.unwrap());
    cache::pubsub::publish(&channel, serde_json::to_vec(&reply).unwrap())
        .await
        .unwrap();
    consumer.ack(&consumed).await.unwrap();

    let echoed = handle.await.unwrap();
    assert_eq!(echoed, 42);
}
