//! Live classes UI: schedule (instructor/admin), browse, and join real-time
//! sessions backed by the `live_classes` plugin. Joining hands off into
//! [`crate::live_room::LiveRoom`] for the actual video/chat/whiteboard.

use dioxus::prelude::*;

use crate::components::*;
use crate::course_types::CourseView;
use crate::dashboard::UiCtx;
use crate::live_room::{join_session, start_session, end_session, LiveRoom, LiveRoomState};
use crate::live_types::{status_label, status_tone, LiveSessionView};
use crate::server_functions::{courses_list, live_cancel, live_list, live_schedule};

#[component]
pub fn LiveClassesPage(can_manage: bool) -> Element {
    let ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());
    let display_name = (ctx.session)().map(|s| s.user.full_name.clone()).unwrap_or_default();
    let mut refresh = use_signal(|| 0u32);
    let mut selected_course: Signal<Option<uuid::Uuid>> = use_signal(|| None);
    let mut show_scheduler = use_signal(|| false);
    let mut room: Signal<Option<LiveRoomState>> = use_signal(|| None);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let token_fetch = token.clone();
    let mut courses = use_resource(move || {
        let token = token_fetch.clone();
        async move {
            match token {
                Some(t) => courses_list(t, if can_manage { None } else { Some("published".into()) }, can_manage, None)
                    .await
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });
    let courses_ref = courses.read();
    let courses_opt: &Option<Vec<CourseView>> = &*courses_ref;

    // Auto-select the first course once the list loads.
    use_effect(move || {
        if selected_course().is_none() {
            if let Some(list) = &*courses.read() {
                if let Some(first) = list.first() {
                    selected_course.set(Some(first.id));
                }
            }
        }
    });

    let token_fetch2 = token.clone();
    let mut sessions = use_resource(move || {
        let token = token_fetch2.clone();
        let course_id = selected_course();
        async move {
            let _ = refresh();
            match (token, course_id) {
                (Some(t), Some(cid)) => live_list(t, cid).await.unwrap_or_default(),
                _ => Vec::new(),
            }
        }
    });
    let sessions_ref = sessions.read();
    let sessions_opt: &Option<Vec<LiveSessionView>> = &*sessions_ref;

    if let Some(state) = room() {
        return rsx! {
            LiveRoom {
                state: state.clone(),
                on_leave: move |_| {
                    room.set(None);
                    refresh += 1;
                },
            }
        };
    }

    let do_join = {
        let token = token.clone();
        let display_name = display_name.clone();
        move |session_id: uuid::Uuid| {
            let token = token.clone().unwrap_or_default();
            let display_name = display_name.clone();
            let mut room = room;
            let mut error = error;
            spawn(async move {
                match join_session(token.clone(), session_id).await {
                    Ok(join) => room.set(Some(LiveRoomState { join, token, display_name })),
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        }
    };

    let do_start = {
        let token = token.clone();
        move |session_id: uuid::Uuid| {
            let token = token.clone().unwrap_or_default();
            let mut refresh = refresh;
            let mut error = error;
            spawn(async move {
                if let Err(e) = start_session(token, session_id).await {
                    error.set(Some(e.to_string()));
                } else {
                    refresh += 1;
                }
            });
        }
    };

    let do_end = {
        let token = token.clone();
        move |session_id: uuid::Uuid| {
            let token = token.clone().unwrap_or_default();
            let mut refresh = refresh;
            let mut error = error;
            spawn(async move {
                if let Err(e) = end_session(token, session_id).await {
                    error.set(Some(e.to_string()));
                } else {
                    refresh += 1;
                }
            });
        }
    };

    let do_cancel = {
        let token = token.clone();
        move |session_id: uuid::Uuid| {
            let token = token.clone().unwrap_or_default();
            let mut refresh = refresh;
            let mut error = error;
            spawn(async move {
                if let Err(e) = live_cancel(token, session_id).await {
                    error.set(Some(e.to_string()));
                } else {
                    refresh += 1;
                }
            });
        }
    };

    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Live classes",
                subtitle: "Join real-time sessions with video, chat, screen share and a shared whiteboard",
                action: rsx! {
                    if can_manage {
                        Button {
                            variant: "filled",
                            icon: "add",
                            onclick: move |_| show_scheduler.set(true),
                            "Schedule session"
                        }
                    }
                    Button { variant: "tonal", icon: "refresh", onclick: move |_| refresh += 1, "Refresh" }
                },
            }
            if let Some(err) = error() {
                div {
                    class: "md-card placeholder",
                    style: "border-color:#e8341a",
                    p { "{err}" }
                }
            }
            div { class: "table-toolbar",
                span { "Course:" }
                select {
                    class: "md-input md-input-sm",
                    onchange: move |e| {
                        if let Ok(id) = uuid::Uuid::parse_str(&e.value()) {
                            selected_course.set(Some(id));
                        }
                    },
                    if let Some(list) = courses_opt {
                        for c in list.iter() {
                            option { value: "{c.id}", "{c.title}" }
                        }
                    }
                }
            }
            if show_scheduler() {
                if let Some(course_id) = selected_course() {
                    ScheduleForm {
                        course_id,
                        on_close: move |_| show_scheduler.set(false),
                        on_saved: move |_| {
                            show_scheduler.set(false);
                            refresh += 1;
                        },
                    }
                }
            }
            if let Some(list) = sessions_opt {
                if list.is_empty() {
                    div { class: "md-card placeholder",
                        span { class: "material-symbols-outlined placeholder-icon", "video_camera_front" }
                        h3 { "No live sessions" }
                        p { "Nothing scheduled for this course yet." }
                    }
                } else {
                    div { class: "md-card table-card",
                        table { class: "md-table",
                            thead {
                                tr {
                                    th { "Title" }
                                    th { "Provider" }
                                    th { "Scheduled" }
                                    th { "Status" }
                                    th { "Actions" }
                                }
                            }
                            tbody {
                                for s in list.iter().cloned() {
                                    tr {
                                        td { "{s.title}" }
                                        td { "{s.provider}" }
                                        td { "{s.scheduled_start}" }
                                        td {
                                            StatusChip {
                                                status: status_tone(s.status.as_str()).to_string(),
                                                label: status_label(s.status.as_str()).to_string(),
                                            }
                                        }
                                        td {
                                            if s.status == "scheduled" && can_manage {
                                                Button {
                                                    variant: "tonal",
                                                    icon: "play_arrow",
                                                    onclick: {
                                                        let f = do_start.clone();
                                                        let id = s.id;
                                                        move |_| f(id)
                                                    },
                                                    "Start"
                                                }
                                            }
                                            if s.status == "live" {
                                                Button {
                                                    variant: "filled",
                                                    icon: "video_call",
                                                    onclick: {
                                                        let f = do_join.clone();
                                                        let id = s.id;
                                                        move |_| f(id)
                                                    },
                                                    "Join"
                                                }
                                            }
                                            if s.status == "scheduled" && !can_manage {
                                                Button {
                                                    variant: "outlined",
                                                    icon: "video_call",
                                                    onclick: {
                                                        let f = do_join.clone();
                                                        let id = s.id;
                                                        move |_| f(id)
                                                    },
                                                    "Preview"
                                                }
                                            }
                                            if s.status == "live" && can_manage {
                                                Button {
                                                    variant: "text",
                                                    icon: "call_end",
                                                    onclick: {
                                                        let f = do_end.clone();
                                                        let id = s.id;
                                                        move |_| f(id)
                                                    },
                                                    "End"
                                                }
                                            }
                                            if s.status == "scheduled" && can_manage {
                                                Button {
                                                    variant: "text",
                                                    icon: "close",
                                                    onclick: {
                                                        let f = do_cancel.clone();
                                                        let id = s.id;
                                                        move |_| f(id)
                                                    },
                                                    "Cancel"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "md-card placeholder",
                    span { class: "material-symbols-outlined placeholder-icon", "hourglass_empty" }
                    h3 { "Loading sessions…" }
                }
            }
        }
    }
}

#[component]
fn ScheduleForm(course_id: uuid::Uuid, on_close: EventHandler<()>, on_saved: EventHandler<()>) -> Element {
    let ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone()).unwrap_or_default();
    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut start = use_signal(String::new);
    let mut end = use_signal(String::new);
    let mut provider = use_signal(|| "webrtc".to_string());
    let mut external_url = use_signal(String::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let save = move |_| {
        let token = token.clone();
        let title = title();
        let description = description();
        let start = start();
        let end = end();
        let provider = provider();
        let external_url = external_url();
        let mut error = error;
        spawn(async move {
            let ext = if external_url.trim().is_empty() { None } else { Some(external_url) };
            match live_schedule(token, course_id, title, description, start, end, provider, ext).await {
                Ok(_) => on_saved.call(()),
                Err(e) => error.set(Some(e.to_string())),
            }
        });
    };

    rsx! {
        div { class: "md-card",
            div { class: "card-title-row",
                h3 { "Schedule a live session" }
            }
            if let Some(err) = error() {
                p { style: "color:#e8341a", "{err}" }
            }
            div { class: "form-grid",
                label { "Title" }
                input {
                    class: "md-input",
                    value: title(),
                    oninput: move |e| title.set(e.value()),
                }
                label { "Description" }
                input {
                    class: "md-input",
                    value: description(),
                    oninput: move |e| description.set(e.value()),
                }
                label { "Start (RFC3339)" }
                input {
                    class: "md-input",
                    placeholder: "2026-09-01T09:00:00Z",
                    value: start(),
                    oninput: move |e| start.set(e.value()),
                }
                label { "End (RFC3339)" }
                input {
                    class: "md-input",
                    placeholder: "2026-09-01T10:00:00Z",
                    value: end(),
                    oninput: move |e| end.set(e.value()),
                }
                label { "Provider" }
                select {
                    class: "md-input",
                    onchange: move |e| provider.set(e.value()),
                    option { value: "webrtc", selected: true, "In-app (WebRTC)" }
                    option { value: "jitsi", "Jitsi" }
                    option { value: "zoom", "Zoom" }
                    option { value: "teams", "Microsoft Teams" }
                }
                if provider() != "webrtc" {
                    label { "External meeting URL" }
                    input {
                        class: "md-input",
                        value: external_url(),
                        oninput: move |e| external_url.set(e.value()),
                    }
                }
            }
            div { class: "table-toolbar",
                Button { variant: "filled", icon: "check", onclick: save, "Schedule" }
                Button {
                    variant: "text",
                    icon: "close",
                    onclick: move |_| on_close.call(()),
                    "Cancel"
                }
            }
        }
    }
}
