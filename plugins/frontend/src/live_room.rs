//! Real-time live classroom: video (WebRTC full-mesh), chat, screen share and
//! a shared whiteboard, backed by the `live_classes` plugin (durable state)
//! and the frontend's `/ws/live/{session_id}` signaling relay (real-time
//! transport). See `live_signaling.rs` for the server side of the protocol.
//!
//! All browser APIs (getUserMedia/getDisplayMedia/RTCPeerConnection/Canvas)
//! are driven from `LIVE_ROOM_JS`, evaluated once per room via
//! `document::eval`, mirroring the existing `LOCAL_MEDIA_JS` pattern in
//! `mock_data.rs`. Rust owns persistence (chat/session lifecycle via
//! `server_functions`); JS owns only the real-time media/relay plumbing.

use dioxus::prelude::*;

use crate::components::*;
use crate::live_types::LiveJoinView;
use crate::server_functions::{live_chat_history, live_end, live_join, live_leave, live_start};

/// Everything the room needs once the caller has joined a session.
#[derive(Clone, PartialEq)]
pub struct LiveRoomState {
    pub join: LiveJoinView,
    pub token: String,
    pub display_name: String,
}

#[component]
pub fn LiveRoom(state: LiveRoomState, on_leave: EventHandler<()>) -> Element {
    let join = state.join.clone();
    let session = join.session.clone();
    let is_host = join.role == "host";

    let mut mic_on = use_signal(|| true);
    let mut cam_on = use_signal(|| true);
    let mut share_on = use_signal(|| false);
    let mut chat_open = use_signal(|| true);
    let mut draft = use_signal(String::new);
    let mut whiteboard_open = use_signal(|| false);
    let mut tool_color = use_signal(|| "#1a73e8".to_string());
    let mut chat: Signal<Vec<(String, String, String)>> = use_signal(Vec::new); // (name, body, iso_time)

    // Load persisted chat history once, then let the WS relay append live messages.
    {
        let session_id = session.id;
        let token = state.token.clone();
        let mut chat = chat;
        use_future(move || {
            let token = token.clone();
            async move {
                if let Ok(history) = live_chat_history(token, session_id).await {
                    chat.set(
                        history
                            .into_iter()
                            .map(|m| (m.sender_id.to_string(), m.body, m.created_at.to_rfc3339()))
                            .collect(),
                    );
                }
            }
        });
    }

    // Connect the signaling WebSocket + local media once the room mounts.
    {
        let signaling_path = join.signaling_path.clone();
        let attendance_id = join.attendance_id;
        let token = state.token.clone();
        let role = join.role.clone();
        let name = state.display_name.clone();
        use_effect(move || {
            if let Some(path) = &signaling_path {
                let js = format!(
                    "window.ElupaneLive && window.ElupaneLive.init({}, {}, {}, {}, {});",
                    js_str(path),
                    js_str(&token),
                    js_str(&role),
                    js_str(&name),
                    js_str(&attendance_id.to_string()),
                );
                let _ = dioxus::document::eval(&format!("{LIVE_ROOM_JS}\n{js}"));
            }
        });
    }

    // Poll for incoming chat/whiteboard relay events pushed onto
    // `window.__eliveChatInbox` by the JS side (simplest bridge back to Rust
    // without a bidirectional eval channel per message).
    {
        let mut chat = chat;
        use_future(move || async move {
            loop {
                let eval = dioxus::document::eval(
                    "return (window.__eliveChatInbox || []).splice(0, 1000);",
                );
                if let Ok(batch) = eval.join::<Vec<serde_json::Value>>().await {
                    if !batch.is_empty() {
                        let mut items = chat();
                        for entry in batch {
                            let name = entry.get("name").and_then(|v| v.as_str()).unwrap_or("Someone").to_string();
                            let body = entry.get("body").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            items.push((name, body, chrono::Utc::now().to_rfc3339()));
                        }
                        chat.set(items);
                    }
                }
                gloo_timers_sleep().await;
            }
        });
    }

    let leave = {
        let token = state.token.clone();
        let attendance_id = join.attendance_id;
        move |_| {
            let token = token.clone();
            let _ = dioxus::document::eval("window.ElupaneLive && window.ElupaneLive.leave();");
            spawn(async move {
                let _ = live_leave(token, attendance_id).await;
            });
            on_leave.call(());
        }
    };

    let toggle_mic = move |_| {
        mic_on.set(!mic_on());
        let _ = dioxus::document::eval(&format!(
            "window.ElupaneLive && window.ElupaneLive.setMic({});",
            mic_on()
        ));
    };
    let toggle_cam = move |_| {
        cam_on.set(!cam_on());
        let _ = dioxus::document::eval(&format!(
            "window.ElupaneLive && window.ElupaneLive.setCam({});",
            cam_on()
        ));
    };
    let toggle_share = move |_| {
        share_on.set(!share_on());
        let _ = dioxus::document::eval("window.ElupaneLive && window.ElupaneLive.toggleShare();");
    };
    let clear_board = move |_| {
        let _ = dioxus::document::eval("window.ElupaneLive && window.ElupaneLive.whiteboardClear();");
    };
    let set_color = move |c: &'static str| {
        let mut tool_color = tool_color;
        tool_color.set(c.to_string());
        let _ = dioxus::document::eval(&format!(
            "window.ElupaneLive && window.ElupaneLive.setTool({});",
            js_str(c)
        ));
    };

    let mut send_chat = {
        let token = state.token.clone();
        let session_id = session.id;
        let display_name = state.display_name.clone();
        move || {
            let body = draft();
            if body.trim().is_empty() {
                return;
            }
            draft.set(String::new());
            let _ = dioxus::document::eval(&format!(
                "window.ElupaneLive && window.ElupaneLive.sendChat({});",
                js_str(&body)
            ));
            let mut items = chat();
            items.push((display_name.clone(), body.clone(), chrono::Utc::now().to_rfc3339()));
            chat.set(items);
            let token = token.clone();
            spawn(async move {
                let _ = crate::server_functions::live_chat_send(token, session_id, body).await;
            });
        }
    };

    rsx! {
        div { class: "page",
            div { class: "live-room",
                div { class: "live-room-head",
                    Button {
                        variant: "text",
                        icon: "arrow_back",
                        onclick: {
                            let f = leave.clone();
                            move |e| f(e)
                        },
                        "Leave"
                    }
                    div { class: "live-room-meta",
                        h2 { "{session.title}" }
                        div { class: "live-room-sub", "{session.description}" }
                    }
                    div { class: "md-topbar-spacer" }
                    span { class: "live-badge",
                        span { class: "pulse-dot" }
                        "LIVE"
                    }
                }

                div { class: "live-layout",
                    div { class: "live-stage",
                        div { class: "video-grid", id: "elive-videos",
                            div { class: "video-tile local",
                                video {
                                    id: "elive-local-video",
                                    autoplay: true,
                                    muted: true,
                                    style: format!(
                                        "width:100%;height:100%;object-fit:cover;display:{}",
                                        if cam_on() { "block" } else { "none" },
                                    ),
                                }
                                span { class: "video-tile-name", "{state.display_name} (You)" }
                            }
                        }
                        if whiteboard_open() {
                            div { class: "whiteboard-panel",
                                div { class: "whiteboard-toolbar",
                                    for c in ["#1a73e8", "#e8341a", "#12a150", "#111111"] {
                                        button {
                                            class: "swatch",
                                            style: format!("background:{c}"),
                                            onclick: {
                                                let f = set_color.clone();
                                                move |_| f(c)
                                            },
                                        }
                                    }
                                    Button {
                                        variant: "text",
                                        icon: "delete",
                                        onclick: clear_board,
                                        "Clear"
                                    }
                                }
                                canvas {
                                    id: "elive-whiteboard",
                                    width: "960",
                                    height: "540",
                                }
                            }
                        }
                    }

                    if chat_open() {
                        div { class: "live-sidebar",
                            Card {
                                div { class: "card-title-row",
                                    h3 { "Class chat" }
                                }
                                div { class: "live-chat",
                                    div { class: "chat-list",
                                        for (name , body , _t) in chat().iter() {
                                            div { class: "chat-msg received",
                                                div { class: "chat-msg-name", "{name}" }
                                                div { class: "chat-msg-text", "{body}" }
                                            }
                                        }
                                    }
                                    div { class: "composer",
                                        input {
                                            class: "composer-input",
                                            placeholder: "Send a message…",
                                            value: draft(),
                                            oninput: move |e| draft.set(e.value()),
                                            onkeydown: {
                                                let mut send_chat = send_chat.clone();
                                                move |e| {
                                                    if e.key() == Key::Enter {
                                                        send_chat();
                                                    }
                                                }
                                            },
                                        }
                                        Button {
                                            icon: "send",
                                            onclick: {
                                                let mut f = send_chat.clone();
                                                move |_| f()
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "live-controls",
                    button {
                        class: format!("ctrl-btn {}", if mic_on() { "on" } else { "off" }),
                        onclick: toggle_mic,
                        title: "Toggle microphone",
                        span { class: "material-symbols-outlined",
                            if mic_on() {
                                "mic"
                            } else {
                                "mic_off"
                            }
                        }
                    }
                    button {
                        class: format!("ctrl-btn {}", if cam_on() { "on" } else { "off" }),
                        onclick: toggle_cam,
                        title: "Toggle camera",
                        span { class: "material-symbols-outlined",
                            if cam_on() {
                                "videocam"
                            } else {
                                "videocam_off"
                            }
                        }
                    }
                    button {
                        class: format!("ctrl-btn {}", if share_on() { "on" } else { "" }),
                        onclick: toggle_share,
                        title: "Share screen",
                        span { class: "material-symbols-outlined",
                            if share_on() {
                                "stop_screen_share"
                            } else {
                                "present_to_all"
                            }
                        }
                    }
                    button {
                        class: format!("ctrl-btn {}", if whiteboard_open() { "on" } else { "" }),
                        onclick: move |_| whiteboard_open.set(!whiteboard_open()),
                        title: "Toggle whiteboard",
                        span { class: "material-symbols-outlined", "draw" }
                    }
                    button {
                        class: format!("ctrl-btn {}", if chat_open() { "on" } else { "" }),
                        onclick: move |_| chat_open.set(!chat_open()),
                        title: "Toggle chat",
                        span { class: "material-symbols-outlined", "chat" }
                    }
                    div { class: "md-topbar-spacer" }
                    if is_host {
                        Button {
                            class: "md-btn-danger",
                            icon: "call_end",
                            onclick: {
                                let f = leave.clone();
                                move |e| f(e)
                            },
                            "End session"
                        }
                    } else {
                        Button {
                            class: "md-btn-danger",
                            icon: "call_end",
                            onclick: {
                                let f = leave.clone();
                                move |e| f(e)
                            },
                            "Leave"
                        }
                    }
                }
            }
        }
    }
}

/// Host action: start a scheduled session (transitions it to `live`).
pub async fn start_session(token: String, session_id: uuid::Uuid) -> anyhow::Result<()> {
    live_start(token, session_id).await?;
    Ok(())
}

/// Host action: end a live session.
pub async fn end_session(token: String, session_id: uuid::Uuid) -> anyhow::Result<()> {
    live_end(token, session_id).await?;
    Ok(())
}

/// Join a session by id, returning everything the room needs.
pub async fn join_session(token: String, session_id: uuid::Uuid) -> anyhow::Result<LiveJoinView> {
    live_join(token, session_id).await
}

fn js_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

/// A tiny cooperative delay so the chat-inbox poller doesn't busy-loop.
/// `document::eval` round-trips are async, so this just paces the polling
/// interval (~400ms) using a JS `setTimeout` bridged back through eval.
async fn gloo_timers_sleep() {
    let eval = dioxus::document::eval(
        "return await new Promise(function(resolve){ setTimeout(resolve, 400); });",
    );
    let _ = eval.join::<serde_json::Value>().await;
}

/// Browser-side WebRTC mesh + whiteboard + chat relay, driven entirely by
/// events over `/ws/live/{session_id}`. Exposed as `window.ElupaneLive`.
const LIVE_ROOM_JS: &str = r#"(function(){
  if (window.ElupaneLive) { return; }
  const state = {
    ws: null, pcs: {}, localStream: null, shareStream: null,
    myId: null, tool: { color: '#1a73e8', size: 3 },
    drawing: false, last: null,
  };
  window.__eliveChatInbox = window.__eliveChatInbox || [];

  function iceServers() { return [{ urls: 'stun:stun.l.google.com:19302' }]; }

  function videoTileFor(peerId, name) {
    let tile = document.getElementById('elive-tile-' + peerId);
    if (tile) return tile.querySelector('video');
    tile = document.createElement('div');
    tile.className = 'video-tile';
    tile.id = 'elive-tile-' + peerId;
    const v = document.createElement('video');
    v.autoplay = true; v.playsInline = true;
    v.style.cssText = 'width:100%;height:100%;object-fit:cover;display:block';
    const label = document.createElement('span');
    label.className = 'video-tile-name';
    label.textContent = name || 'Participant';
    tile.appendChild(v); tile.appendChild(label);
    const grid = document.getElementById('elive-videos');
    if (grid) grid.appendChild(tile);
    return v;
  }

  function removeTile(peerId) {
    const tile = document.getElementById('elive-tile-' + peerId);
    if (tile && tile.parentNode) tile.parentNode.removeChild(tile);
  }

  function send(obj) { if (state.ws && state.ws.readyState === 1) state.ws.send(JSON.stringify(obj)); }

  function createPeer(peerId, name, initiator) {
    if (state.pcs[peerId]) return state.pcs[peerId];
    const pc = new RTCPeerConnection({ iceServers: iceServers() });
    state.pcs[peerId] = pc;
    if (state.localStream) {
      state.localStream.getTracks().forEach(function (t) { pc.addTrack(t, state.localStream); });
    }
    pc.onicecandidate = function (e) {
      if (e.candidate) send({ type: 'signal', to: peerId, data: { kind: 'candidate', candidate: e.candidate } });
    };
    pc.ontrack = function (e) {
      const v = videoTileFor(peerId, name);
      v.srcObject = e.streams[0];
    };
    if (initiator) {
      pc.createOffer().then(function (offer) {
        return pc.setLocalDescription(offer);
      }).then(function () {
        send({ type: 'signal', to: peerId, data: { kind: 'offer', sdp: pc.localDescription } });
      });
    }
    return pc;
  }

  function handleSignal(msg) {
    if (msg.to && state.myId && msg.to !== state.myId) return;
    const peerId = msg.from; const data = msg.data || {};
    const pc = createPeer(peerId, msg.name, false);
    if (data.kind === 'offer') {
      pc.setRemoteDescription(data.sdp).then(function () {
        return pc.createAnswer();
      }).then(function (answer) {
        return pc.setLocalDescription(answer);
      }).then(function () {
        send({ type: 'signal', to: peerId, data: { kind: 'answer', sdp: pc.localDescription } });
      });
    } else if (data.kind === 'answer') {
      pc.setRemoteDescription(data.sdp);
    } else if (data.kind === 'candidate') {
      pc.addIceCandidate(data.candidate).catch(function () {});
    }
  }

  function drawSegment(op) {
    const canvas = document.getElementById('elive-whiteboard');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (op.clear) { ctx.clearRect(0, 0, canvas.width, canvas.height); return; }
    ctx.strokeStyle = op.color || '#1a73e8';
    ctx.lineWidth = op.size || 3;
    ctx.lineCap = 'round';
    ctx.beginPath();
    ctx.moveTo(op.x0 * canvas.width, op.y0 * canvas.height);
    ctx.lineTo(op.x1 * canvas.width, op.y1 * canvas.height);
    ctx.stroke();
  }

  function wireWhiteboard() {
    const canvas = document.getElementById('elive-whiteboard');
    if (!canvas || canvas.__wired) return;
    canvas.__wired = true;
    function pos(e) {
      const r = canvas.getBoundingClientRect();
      const cx = (e.touches ? e.touches[0].clientX : e.clientX) - r.left;
      const cy = (e.touches ? e.touches[0].clientY : e.clientY) - r.top;
      return { x: cx / r.width, y: cy / r.height };
    }
    function start(e) { state.drawing = true; state.last = pos(e); }
    function move(e) {
      if (!state.drawing) return;
      const p = pos(e);
      const op = { x0: state.last.x, y0: state.last.y, x1: p.x, y1: p.y, color: state.tool.color, size: state.tool.size };
      drawSegment(op);
      send({ type: 'whiteboard', op: op });
      state.last = p;
    }
    function stop() { state.drawing = false; }
    canvas.addEventListener('mousedown', start);
    canvas.addEventListener('mousemove', move);
    window.addEventListener('mouseup', stop);
    canvas.addEventListener('touchstart', start);
    canvas.addEventListener('touchmove', move);
    canvas.addEventListener('touchend', stop);
  }

  window.ElupaneLive = {
    init: function (path, token, role, name, attendanceId) {
      if (state.ws) return; // already initialised for this room
      wireWhiteboard();
      navigator.mediaDevices.getUserMedia({ video: { width: 1280 }, audio: true }).then(function (stream) {
        state.localStream = stream;
        const v = document.getElementById('elive-local-video');
        if (v) v.srcObject = stream;
        const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
        let url = proto + '//' + location.host + path + '?token=' + encodeURIComponent(token) + '&role=' + encodeURIComponent(role);
        if (attendanceId) url += '&attendance_id=' + encodeURIComponent(attendanceId);
        const ws = new WebSocket(url);
        state.ws = ws;
        ws.onmessage = function (e) {
          let msg; try { msg = JSON.parse(e.data); } catch (err) { return; }
          if (msg.type === 'peers') {
            (msg.peers || []).forEach(function (p) { createPeer(p.peer_id, p.name, true); });
          } else if (msg.type === 'hello') {
            if (msg.peer_id) { state.myId = state.myId || null; createPeer(msg.peer_id, msg.name, false); }
          } else if (msg.type === 'signal') {
            handleSignal(msg);
          } else if (msg.type === 'chat') {
            window.__eliveChatInbox.push({ name: msg.name, body: msg.body });
          } else if (msg.type === 'whiteboard') {
            drawSegment(msg.op || {});
          } else if (msg.type === 'leave') {
            if (state.pcs[msg.peer_id]) { state.pcs[msg.peer_id].close(); delete state.pcs[msg.peer_id]; }
            removeTile(msg.peer_id);
          }
        };
      }).catch(function () {});
    },
    setMic: function (on) { if (state.localStream) state.localStream.getAudioTracks().forEach(function (t) { t.enabled = on; }); },
    setCam: function (on) { if (state.localStream) state.localStream.getVideoTracks().forEach(function (t) { t.enabled = on; }); },
    toggleShare: function () {
      if (state.shareStream) {
        state.shareStream.getTracks().forEach(function (t) { t.stop(); });
        state.shareStream = null;
        const camTrack = state.localStream && state.localStream.getVideoTracks()[0];
        Object.values(state.pcs).forEach(function (pc) {
          const sender = pc.getSenders().find(function (s) { return s.track && s.track.kind === 'video'; });
          if (sender && camTrack) sender.replaceTrack(camTrack);
        });
        return;
      }
      navigator.mediaDevices.getDisplayMedia({ video: true }).then(function (stream) {
        state.shareStream = stream;
        const shareTrack = stream.getVideoTracks()[0];
        Object.values(state.pcs).forEach(function (pc) {
          const sender = pc.getSenders().find(function (s) { return s.track && s.track.kind === 'video'; });
          if (sender) sender.replaceTrack(shareTrack);
        });
        shareTrack.addEventListener('ended', function () { window.ElupaneLive.toggleShare(); });
      }).catch(function () {});
    },
    sendChat: function (body) { send({ type: 'chat', body: body }); },
    whiteboardClear: function () { send({ type: 'whiteboard', op: { clear: true } }); drawSegment({ clear: true }); },
    setTool: function (color) { state.tool.color = color; },
    leave: function () {
      Object.values(state.pcs).forEach(function (pc) { pc.close(); });
      state.pcs = {};
      if (state.localStream) state.localStream.getTracks().forEach(function (t) { t.stop(); });
      if (state.shareStream) state.shareStream.getTracks().forEach(function (t) { t.stop(); });
      if (state.ws) { state.ws.close(); state.ws = null; }
    },
  };
})();"#;
