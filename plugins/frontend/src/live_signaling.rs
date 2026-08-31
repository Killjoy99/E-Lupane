//! Real-time signaling relay for `webrtc`-provider live sessions.
//!
//! The browser never talks to Redis or the `live_classes` plugin directly for
//! real-time traffic — durable state (scheduling, attendance, chat history)
//! goes through [`crate::live_bridge`] over Redis RPC, but low-latency,
//! session-scoped traffic (WebRTC offer/answer/ICE candidates, chat relay,
//! whiteboard strokes) is relayed in-process over a plain WebSocket exposed
//! directly by this Axum server:
//!
//! ```text
//! browser A ─┐                                   ┌─ browser B
//!             ├─ WS /ws/live/{session_id} ────────┤
//! browser C ─┘      (this module, in-process)     └─ browser D
//! ```
//!
//! Peer discovery is full-mesh: every participant opens one `RTCPeerConnection`
//! per other participant, using this channel purely to exchange SDP offers/
//! answers and ICE candidates (see `LOCAL_MEDIA_JS` friends in `mock_data.rs`,
//! extended in `live_room.rs`). This keeps the design provider-agnostic per
//! `AGENTS/live_classes.md`: swapping in Jitsi/Zoom/Teams only means the
//! `live_classes` plugin returns a different `provider`/`external_url`, and
//! the frontend skips this relay entirely (see `LiveJoinView::signaling_path`).
//!
//! Not durable and not distributed: state lives in this process's memory, so
//! this only works for a single frontend instance. Running multiple frontend
//! replicas behind a load balancer would need this relay to fan out over
//! Redis Pub/Sub instead (tracked as a known limitation in the live_classes
//! plugin README).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use dioxus::server::axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use dioxus::server::axum::extract::{Path, Query};
use dioxus::server::axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::broadcast;
use uuid::Uuid;

/// Per-room fan-out channel + membership, keyed by `session_id`.
struct Room {
    tx: broadcast::Sender<String>,
    peers: Mutex<HashMap<Uuid, PeerInfo>>,
}

#[derive(Clone, serde::Serialize)]
struct PeerInfo {
    peer_id: Uuid,
    name: String,
    role: String,
}

static ROOMS: OnceLock<Mutex<HashMap<Uuid, std::sync::Arc<Room>>>> = OnceLock::new();

fn room_for(session_id: Uuid) -> std::sync::Arc<Room> {
    let rooms = ROOMS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut rooms = rooms.lock().expect("rooms mutex poisoned");
    rooms
        .entry(session_id)
        .or_insert_with(|| {
            let (tx, _rx) = broadcast::channel(256);
            std::sync::Arc::new(Room { tx, peers: Mutex::new(HashMap::new()) })
        })
        .clone()
}

#[derive(Debug, Deserialize)]
pub struct WsParams {
    /// Bearer token (verified against the auth plugin) — browsers can't set
    /// custom headers on a WebSocket handshake, so it travels as a query param.
    token: String,
    /// "host" or "attendee", as returned by `live.join`.
    #[serde(default = "default_role")]
    role: String,
    /// The attendance row id from `live.join`, so we can stamp `left_at` when
    /// the socket closes.
    #[serde(default)]
    attendance_id: Option<Uuid>,
}

fn default_role() -> String {
    "attendee".to_string()
}

/// `GET /ws/live/:session_id?token=...&role=...&attendance_id=...`
pub async fn ws_handler(
    Path(session_id): Path<Uuid>,
    Query(params): Query<WsParams>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let user = match crate::auth_bridge::verify(&params.token).await {
        Ok(Some(user)) => user,
        _ => return (axum_unauthorized()).into_response(),
    };
    let role = if params.role == "host" { "host" } else { "attendee" }.to_string();
    let token = params.token.clone();
    ws.on_upgrade(move |socket| {
        handle_socket(socket, session_id, user.id, user.full_name, role, params.attendance_id, token)
    })
}

fn axum_unauthorized() -> dioxus::server::axum::http::StatusCode {
    dioxus::server::axum::http::StatusCode::UNAUTHORIZED
}

async fn handle_socket(
    socket: WebSocket,
    session_id: Uuid,
    user_id_str: String,
    name: String,
    role: String,
    attendance_id: Option<Uuid>,
    token: String,
) {
    let peer_id = Uuid::parse_str(&user_id_str).unwrap_or_else(|_| Uuid::new_v4());
    let room = room_for(session_id);

    // Snapshot existing peers for the newcomer, then register ourselves.
    let existing: Vec<PeerInfo> = {
        let mut peers = room.peers.lock().expect("peers mutex poisoned");
        let existing = peers.values().cloned().collect();
        peers.insert(peer_id, PeerInfo { peer_id, name: name.clone(), role: role.clone() });
        existing
    };

    let (mut ws_tx, mut ws_rx) = socket.split();
    let mut room_rx = room.tx.subscribe();

    // Tell the newcomer who's already here.
    let peers_msg = json!({ "type": "peers", "peers": existing }).to_string();
    if ws_tx.send(WsMessage::Text(peers_msg.into())).await.is_err() {
        room.peers.lock().expect("peers mutex poisoned").remove(&peer_id);
        return;
    }

    // Announce the newcomer to everyone else already in the room.
    let _ = room.tx.send(
        json!({ "type": "hello", "peer_id": peer_id, "name": name, "role": role }).to_string(),
    );

    // Forward the room's broadcast traffic to this socket.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = room_rx.recv().await {
            if ws_tx.send(WsMessage::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Read from this socket, stamp a verified `from`/`name`, and re-broadcast.
    let room_for_recv = room.clone();
    let recv_peer_id = peer_id;
    let recv_name = name.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            if let WsMessage::Text(text) = msg {
                if let Ok(mut value) = serde_json::from_str::<Value>(&text) {
                    if let Some(obj) = value.as_object_mut() {
                        obj.insert("from".to_string(), json!(recv_peer_id));
                        obj.insert("name".to_string(), json!(recv_name));
                    }
                    let _ = room_for_recv.tx.send(value.to_string());
                }
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Cleanup: drop membership, tell everyone else, stamp attendance leave.
    room.peers.lock().expect("peers mutex poisoned").remove(&peer_id);
    let _ = room.tx.send(json!({ "type": "leave", "peer_id": peer_id }).to_string());
    if let Some(attendance_id) = attendance_id {
        let _ = crate::live_bridge::leave(&token, attendance_id).await;
    }
}
