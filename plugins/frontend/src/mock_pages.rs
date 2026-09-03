use dioxus::prelude::*;
use crate::components::*;
use crate::dashboard::Role;
use crate::mock_data::{
    ACTIVITY, ANNOUNCEMENTS, API_GROUPS, APPROVALS, ASSIGNMENTS, AUDIT_LOG, CERTIFICATES,
    CONVERSATIONS, COURSES, DEADLINES, ENGAGEMENT, GRADE_DIST, INSTITUTIONS, LESSONS,
    PERMISSIONS, PERM_MATRIX, PLUGINS, QUIZZES, ROLES, ROLE_ICONS, ROLE_SPLIT, ROSTER,
    SDKS, SERVICES, SUBMISSIONS, TIMETABLE, USERS, WEBHOOKS, WEEKLY_ACTIVITY, WEEKLY_TREND,
};

#[component]
pub fn LiveLesson(id: &'static str, role: Role) -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Live lesson", subtitle: "Signaling relay ready", action: None }
            Placeholder { title: "Live session", desc: format!("WebRTC signaling is active for session {id}."), icon: "play_circle" }
        }
    }
}

#[component]
pub fn StudentMessages() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Messages", subtitle: "Conversations and notifications" },
            div { class: "chat-layout",
                div { class: "chat-sidebar",
                    div { class: "chat-search",
                        span { class: "material-symbols-outlined", "search" }
                        input { placeholder: "Search conversations…", r#type: "text" }
                    }
                    for c in CONVERSATIONS.iter() {
                        div { class: format!("chat-conv {}", if c.unread > 0 { "unread" } else { "" }),
                            div { class: format!("chat-avatar md-avatar-{}", c.tone), "{c.initials}" }
                            div { class: "chat-meta",
                                div { class: "chat-name", "{c.person}" }
                                div { class: "chat-preview", "{c.preview}" }
                            }
                            if c.unread > 0 {
                                div { class: "chat-unread", "{c.unread}" }
                            }
                            div { class: format!("chat-time {}", if c.online { "online" } else { "" }), "{c.time}" }
                        }
                    }
                }
                div { class: "chat-main",
                    div { class: "chat-header",
                        div { class: "chat-participant",
                            span { class: "material-symbols-outlined", "account_circle" }
                            "Active conversation"
                        }
                        div { class: "chat-actions",
                            Button { variant: "text", icon: "videocam", onclick: |_| {}, "Video" }
                            Button { variant: "text", icon: "call", onclick: |_| {}, "Call" }
                            Button { variant: "text", icon: "more_vert", onclick: |_| {}, "" }
                        }
                    }
                    div { class: "chat-messages",
                        for msg in CONVERSATIONS[0].thread.iter() {
                            div { class: format!("msg {}", if msg.sent { "sent" } else { "received" }),
                                div { class: "msg-bubble", "{msg.text}" }
                                div { class: "msg-time", "{msg.time}" }
                            }
                        }
                    }
                    div { class: "chat-input",
                        input { placeholder: "Type a message…", r#type: "text" }
                        Button { variant: "filled", icon: "send", onclick: |_| {}, "" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AdminPlugins() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Plugins", subtitle: "Installed platform plugins" }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { style: "font-weight: 600", "Registered plugins" }
                    Button { variant: "text", icon: "add", onclick: |_| {}, "Register plugin" }
                }
                table { class: "md-table",
                    thead { tr { th { "Plugin" } th { "Language" } th { "Version" } th { "Port" } th { "Signature" } th { "Status" } } }
                    tbody {
                        for p in PLUGINS.iter() {
                            tr {
                                td { div { style: "font-weight: 500", "{p.name}" } }
                                td { class: "muted", "{p.lang}" }
                                td { class: "muted", "{p.version}" }
                                td { class: "muted", "{p.port}" }
                                td {
                                    if p.signed {
                                        StatusChip { status: "signed".to_string(), label: Some("Signed".to_string()) }
                                    } else {
                                        StatusChip { status: "unsigned".to_string(), label: Some("Unsigned".to_string()) }
                                    }
                                }
                                td { StatusChip { status: p.status.to_string(), label: Some(p.status.to_string()) } }
                            }
                        }
                    }
                }
            }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { style: "font-weight: 600", "Pending approvals" }
                }
                table { class: "md-table",
                    thead { tr { th { "Plugin" } th { "Language" } th { "Requester" } th { "Key" } } }
                    tbody {
                        for a in APPROVALS.iter() {
                            tr {
                                td { "{a.plugin}" }
                                td { class: "muted", "{a.lang}" }
                                td { "{a.requester}" }
                                td { class: "muted", "{a.key}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AdminPermissions() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Permissions", subtitle: "RBAC permission matrix" }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { style: "font-weight: 600", "Permission matrix" }
                    Button { variant: "text", icon: "save", onclick: |_| {}, "Save changes" }
                }
                table { class: "perm-table",
                    thead {
                        tr {
                            th { "Permission" }
                            for r in ROLES.iter() { th { "{r}" } }
                        }
                    }
                    tbody {
                        for (i, perm) in PERMISSIONS.iter().enumerate() {
                            tr {
                                td {
                                    span { class: format!("material-symbols-outlined perm-icon"), "{perm.1}" }
                                    "{perm.0}"
                                }
                                for (j, _role) in ROLES.iter().enumerate() {
                                    td {
                                        div { class: format!("perm-cell {}", if perm_granted(i, j) { "on" } else { "" }),
                                            if perm_granted(i, j) { "✓" } else { "" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "perm-actions",
                    div { class: "perm-legend",
                        span { class: "perm-legend-label", "Legend:" }
                        div { class: "perm-cell on" }
                        span { class: "perm-legend-label", "Granted" }
                        div { class: "perm-cell" }
                        span { class: "perm-legend-label", "Denied" }
                    }
                }
            }
        }
    }
}

pub fn student_timetable() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Timetable", subtitle: "Your weekly lectures, labs and tutorials" }
            div { class: "timetable",
                for d in TIMETABLE.iter() {
                    div { class: "tt-day",
                        div { class: "tt-day-head", "{d.day}" }
                        for s in d.slots.iter() {
                            div { class: format!("tt-slot {}", if s.live { "live" } else { "" }),
                                div { class: "tt-time", "{s.time}" }
                                div { "{s.course}" }
                                div { class: "muted", style: "font-size: 11px", "{s.room}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn student_grades() -> Element {
    let dist: Vec<(String, String, String)> = GRADE_DIST
        .iter()
        .map(|(l, v, t)| (
            l.to_string(),
            format!("{:.0}%", v),
            format!("width: {:.0}%; background: var(--tone-{}))", v, t),
        ))
        .collect();
    rsx! {
        div { class: "page",
            SectionTitle { title: "Grades", subtitle: "Assignment results and grade distribution" }
            div { class: "grid-2",
                div { class: "md-card table-card",
                    table { class: "md-table",
                        thead { tr { th { "Assignment" } th { "Course" } th { "Due" } th { "Status" } th { "Grade" } } }
                        tbody {
                            for a in ASSIGNMENTS.iter() {
                                tr {
                                    td { "{a.title}" }
                                    td { class: "muted", "{a.course}" }
                                    td { class: "muted", "{a.due}" }
                                    td { StatusChip { status: a.status.to_string(), label: Some(a.status.to_string()) } }
                                    td {
                                        if let Some(gr) = a.grade {
                                            span { class: "grade-pill", "{gr}" }
                                        } else {
                                            span { class: "muted", "—" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "md-card",
                    div { class: "card-title-row", h3 { "Grade distribution" } }
                    div { class: "h-bars",
                        for (label, val, style) in dist.iter() {
                            div { class: "h-bar-row",
                                div { class: "h-bar-row-head",
                                    span { "{label}" }
                                    span { class: "h-bar-val", "{val}" }
                                }
                                div { class: "h-bar",
                                    div { class: "h-bar-fill", style: "{style}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn student_certificates() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Certificates", subtitle: "Verified credentials earned on E-Lupane" }
            div { class: "grid-3",
                for c in CERTIFICATES.iter() {
                    div { class: "md-card", style: "display: flex; flex-direction: column; align-items: center; text-align: center; gap: 10px; padding: 26px",
                        div { class: "cert-icon",
                            span { class: "material-symbols-outlined", style: format!("font-size: 36px; color: var(--tone-{})", c.tone), "{c.icon}" }
                        }
                        h3 { "{c.title}" }
                        p { class: "muted", "{c.issuer}" }
                        span { class: "mono", "{c.credential}" }
                        p { class: "muted", "{c.date}" }
                        if c.date.contains("Issued") {
                            StatusChip { status: "ok".to_string(), label: Some("Earned".to_string()) }
                        } else {
                            StatusChip { status: "warn".to_string(), label: Some("In progress".to_string()) }
                        }
                    }
                }
            }
        }
    }
}

pub fn student_overview(search: &str) -> Element {
    let _ = search;
    rsx! {
        div { class: "page",
            SectionTitle { title: "Dashboard", subtitle: "Good morning, Thandiwe" }
            div { class: "grid-4",
                StatCard { label: "Enrolled courses", value: format!("{}", COURSES.len()), icon: "menu_book", tone: "teal", delta: Some("3 active") }
                StatCard { label: "Assignments due", value: format!("{}", DEADLINES.len()), icon: "assignment", tone: "indigo", delta: None }
                StatCard { label: "Unread messages", value: format!("{}", CONVERSATIONS.iter().map(|c| c.unread).sum::<u32>()), icon: "chat_bubble", tone: "green", delta: None }
                StatCard { label: "Certificates", value: format!("{}", CERTIFICATES.iter().filter(|c| c.date.contains("Issued")).count()), icon: "workspace_premium", tone: "amber", delta: None }
            }
            div { class: "grid-2",
                div { class: "md-card table-card",
                    div { class: "table-toolbar",
                        span { style: "font-weight: 600", "Weekly activity" }
                    }
                    div { class: "chart-bar",
                        for (label, val, tone) in WEEKLY_ACTIVITY.iter() {
                            div { class: format!("bar-group {tone}"),
                                div { class: "bar", style: format!("height: {:.0}%", val) }
                                div { class: "bar-label", "{label}" }
                            }
                        }
                    }
                }
                div { class: "md-card table-card",
                    div { class: "table-toolbar",
                        span { style: "font-weight: 600", "Upcoming deadlines" }
                    }
                    table { class: "md-table",
                        thead { tr { th { "Assignment" } th { "Course" } th { "When" } th { "Kind" } } }
                        tbody {
                            for d in DEADLINES.iter() {
                                tr {
                                    td { "{d.title}" }
                                    td { class: "muted", "{d.course}" }
                                    td { "{d.when}" }
                                    td { StatusChip { status: d.kind.to_string(), label: Some(d.kind.to_string()) } }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { style: "font-weight: 600", "Recent announcements" }
                }
                for a in ANNOUNCEMENTS.iter().take(3) {
                    div { class: "announcement",
                        span { class: "material-symbols-outlined announcement-icon", "campaign" }
                        div { class: "announcement-body",
                            div { class: "announcement-title", "{a.title}" }
                            div { class: "announcement-text", "{a.body}" }
                            div { class: "announcement-meta", "{a.author} · {a.time}" }
                        }
                    }
                }
            }
        }
    }
}

pub fn lecturer_students() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Students", subtitle: "CS101 · 42 enrolled students" }
            div { class: "grid-2",
                StatCard { label: "On track", value: format!("{}", ROSTER.iter().filter(|r| r.status == "on-track").count()), icon: "trending_up", tone: "green", delta: None }
                StatCard { label: "At risk", value: format!("{}", ROSTER.iter().filter(|r| r.status == "at-risk").count()), icon: "warning", tone: "amber", delta: None }
            }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { style: "font-weight: 600", "Student roster" }
                }
                table { class: "md-table",
                    thead { tr { th { "Student" } th { "Course" } th { "Grade" } th { "Status" } } }
                    tbody {
                        for s in ROSTER.iter() {
                            tr {
                                td {
                                    div { class: "flex-center",
                                        span { class: "material-symbols-outlined avatar-sm", "account_circle" }
                                        "{s.name}"
                                    }
                                }
                                td { class: "muted", "{s.course}" }
                                td { "{s.grade}" }
                                td { StatusChip { status: s.status.to_string(), label: Some(s.status.to_string()) } }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn lecturer_gradebook() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Grade book", subtitle: "Recent submissions requiring your attention" }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { style: "font-weight: 600", "Submissions" }
                    Button { variant: "text", icon: "grade", onclick: |_| {}, "Bulk grade" }
                }
                table { class: "md-table",
                    thead { tr { th { "Student" } th { "Item" } th { "Course" } th { "Submitted" } th { "Status" } } }
                    tbody {
                        for s in SUBMISSIONS.iter() {
                            tr {
                                td { "{s.name}" }
                                td { "{s.item}" }
                                td { class: "muted", "{s.course}" }
                                td { class: "muted", "{s.time}" }
                                td {
                                    if s.state == "new" {
                                        StatusChip { status: "new".to_string(), label: Some("Needs grading".to_string()) }
                                    } else if s.state == "flagged" {
                                        StatusChip { status: "flagged".to_string(), label: Some("Flagged".to_string()) }
                                    } else {
                                        StatusChip { status: s.state.to_string(), label: Some(s.state.to_string()) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { style: "font-weight: 600", "Assignment results" }
                }
                table { class: "md-table",
                    thead { tr { th { "Student" } th { "Course" } th { "Grade" } th { "Status" } } }
                    tbody {
                        for a in ASSIGNMENTS.iter() {
                            if let Some(grade) = a.grade {
                                tr {
                                    td { "{a.title}" }
                                    td { class: "muted", "{a.course}" }
                                    td { "{grade}" }
                                    td { StatusChip { status: a.status.to_string(), label: Some(a.status.to_string()) } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn lecturer_analytics() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Analytics", subtitle: "Course engagement and trends" }
            div { class: "grid-2",
                div { class: "md-card table-card",
                    div { class: "table-toolbar",
                        span { style: "font-weight: 600", "Weekly submissions trend" }
                    }
                    div { class: "chart-bar",
                        for (label, val, tone) in WEEKLY_TREND.iter() {
                            div { class: format!("bar-group {tone}"),
                                div { class: "bar", style: format!("height: {:.0}%", val) }
                                div { class: "bar-label", "{label}" }
                            }
                        }
                    }
                }
                div { class: "md-card table-card",
                    div { class: "table-toolbar",
                        span { style: "font-weight: 600", "Engagement by assignment" }
                    }
                    div { class: "chart-bar",
                        for (label, val, tone) in ENGAGEMENT.iter() {
                            div { class: format!("bar-group {tone}"),
                                div { class: "bar-stacked",
                                    div { class: "bar-fill", style: format!("width: {:.0}%", val) }
                                }
                                div { class: "bar-label", "{label}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn lecturer_overview() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Dashboard", subtitle: "Good morning, Dr. Dlamini" }
            div { class: "grid-4",
                StatCard { label: "Active courses", value: format!("{}", COURSES.len()), icon: "menu_book", tone: "teal", delta: Some("3 published") }
                StatCard { label: "Pending submissions", value: format!("{}", SUBMISSIONS.iter().filter(|s| s.state == "new").count()), icon: "pending_actions", tone: "indigo", delta: None }
                StatCard { label: "Quizzes active", value: format!("{}", QUIZZES.len()), icon: "quiz", tone: "green", delta: None }
                StatCard { label: "Avg. grade", value: "78%".to_string(), icon: "grade", tone: "amber", delta: None }
            }
            div { class: "grid-2",
                div { class: "md-card table-card",
                    div { class: "table-toolbar",
                        span { style: "font-weight: 600", "Upcoming lessons" }
                    }
                    table { class: "md-table",
                        thead { tr { th { "Lesson" } th { "Course" } th { "When" } th { "Students" } } }
                        tbody {
                            for l in LESSONS.iter().filter(|l| l.status != "ended") {
                                tr {
                                    td { "{l.title}" }
                                    td { class: "muted", "{l.course}" }
                                    td { "{l.date} · {l.time}" }
                                    td { "{l.participants}/{l.capacity}" }
                                }
                            }
                        }
                    }
                }
                div { class: "md-card table-card",
                    div { class: "table-toolbar",
                        span { style: "font-weight: 600", "Upcoming deadlines" }
                    }
                    table { class: "md-table",
                        thead { tr { th { "Item" } th { "Course" } th { "When" } } }
                        tbody {
                            for d in DEADLINES.iter() {
                                tr {
                                    td { "{d.title}" }
                                    td { class: "muted", "{d.course}" }
                                    td { "{d.when}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn developer_plugins(search: &str) -> Element {
    let q = search.trim().to_lowercase();
    rsx! {
        div { class: "page",
            SectionTitle { title: "Plugins", subtitle: "Registered plugins, language, signature state and live status" }
            div { class: "md-card table-card",
                table { class: "md-table",
                    thead { tr { th { "Plugin" } th { "Language" } th { "Version" } th { "Port" } th { "Signature" } th { "Status" } } }
                    tbody {
                        for p in PLUGINS.iter().filter(|p| q.is_empty() || p.name.to_lowercase().contains(&q)) {
                            tr {
                                td { "{p.name}" }
                                td { class: "muted", "{p.lang}" }
                                td { class: "muted", "v{p.version}" }
                                td { class: "muted", "{p.port}" }
                                td {
                                    if p.signed {
                                        span { class: "md-status md-status-ok", span { class: "md-status-dot" } "Signed" }
                                    } else {
                                        span { class: "md-status md-status-warn", span { class: "md-status-dot" } "Unsigned" }
                                    }
                                }
                                td { StatusChip { status: p.status.to_string(), label: Some(p.status.to_string()) } }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn developer_sdks() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Plugin SDKs", subtitle: "Toolkits for building, signing and shipping E-Lupane plugins" }
            div { class: "grid-3",
                for s in SDKS.iter() {
                    div { class: "md-card", style: "display: flex; flex-direction: column; gap: 10px; padding: 18px",
                        div { style: "display: flex; align-items: center; gap: 10px",
                            span { class: "material-symbols-outlined", style: "font-size: 26px", "code_blocks" }
                            div {
                                h3 { style: "margin: 0", "{s.lang}" }
                                p { class: "muted", style: "margin: 0; font-size: 13px", "v{s.version}" }
                            }
                        }
                        p { class: "muted", style: "font-size: 13px; flex: 1", "{s.note}" }
                        div { style: "display: flex; align-items: center; justify-content: space-between",
                            StatusChip { status: s.status.to_string(), label: Some(sdk_label(s.status)) }
                            Button { variant: "tonal", icon: "download", onclick: |_| {}, "Get SDK" }
                        }
                    }
                }
            }
        }
    }
}

fn sdk_label(status: &str) -> String {
    match status {
        "available" => "Published".to_string(),
        "review" => "In review".to_string(),
        "pending" => "Planned".to_string(),
        _ => status.to_string(),
    }
}

pub fn developer_system() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "System health", subtitle: "Core services and real-time platform indicators" }
            div { class: "grid-4",
                StatCard { label: "Gateway port", value: "8080".to_string(), icon: "hub", tone: "teal", delta: Some("online") }
                StatCard { label: "Queue lag", value: "2m".to_string(), icon: "hourglass_bottom", tone: "amber", delta: Some("behind") }
                StatCard { label: "Cache hit rate", value: "96.4%".to_string(), icon: "bolt", tone: "indigo", delta: None }
                StatCard { label: "Failed publishes", value: "0".to_string(), icon: "device_hub", tone: "green", delta: None }
            }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { class: "muted", "Live status · refreshed on navigation" }
                    Button { variant: "tonal", icon: "refresh", onclick: |_| {}, "Refresh" }
                }
                table { class: "md-table",
                    thead { tr { th { "Service" } th { "Detail" } th { "Status" } } }
                    tbody {
                        for s in SERVICES.iter() {
                            tr {
                                td { "{s.name}" }
                                td { class: "muted", "{s.detail}" }
                                td { StatusChip { status: s.status.to_string(), label: Some(if s.status == "ok" { "operational".to_string() } else { s.status.to_string() }) } }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn developer_webhooks() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Webhooks", subtitle: "Outbound event deliveries to your integrations" }
            div { class: "md-card table-card",
                table { class: "md-table",
                    thead { tr { th { "Name" } th { "URL" } th { "Events" } th { "Status" } } }
                    tbody {
                        for w in WEBHOOKS.iter() {
                            tr {
                                td { "{w.name}" }
                                td { class: "muted", style: "font-family: monospace; font-size: 12px", "{w.url}" }
                                td { class: "muted", "{w.events}" }
                                td { StatusChip { status: w.status.to_string(), label: Some(w.status.to_string()) } }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn developer_docs(search: &str) -> Element {
    let q = search.trim().to_lowercase();
    rsx! {
        div { class: "page",
            SectionTitle { title: "API reference", subtitle: "Gateway endpoints available to plugins, SDks and external integrations" }
            for g in API_GROUPS.iter() {
                div { class: "md-card table-card", style: "margin-bottom: 14px",
                    div { class: "table-toolbar",
                        span { class: "material-symbols-outlined", "{g.icon}" }
                        span { style: "font-weight: 600", " {g.name}" }
                    }
                    table { class: "md-table",
                        thead { tr { th { "Method" } th { "Path" } th { "Description" } th { "Auth" } } }
                        tbody {
                            for e in g.endpoints.iter().filter(|e| q.is_empty() || e.path.to_lowercase().contains(&q) || e.desc.to_lowercase().contains(&q)) {
                                tr {
                                    td {
                                        span { style: format!("font-weight: 700; font-size: 12px; color: {}", method_tone(e.method)), "{e.method}" }
                                    }
                                    td { style: "font-family: monospace; font-size: 12px", "{e.path}" }
                                    td { class: "muted", "{e.desc}" }
                                    td { class: "muted", "{e.auth}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn method_tone(m: &str) -> String {
    match m {
        "GET" => "#1a7f37".to_string(),
        "POST" => "#1f6feb".to_string(),
        "PUT" => "#9a6700".to_string(),
        "DELETE" => "#cf222e".to_string(),
        _ => "#57606a".to_string(),
    }
}

pub fn DeveloperOverview() -> Element {
    let running = PLUGINS.iter().filter(|p| p.status == "running").count();
    let sdks_out = SDKS.iter().filter(|s| s.status == "available").count();
    let hooks_live = WEBHOOKS.iter().filter(|w| w.status == "healthy").count();
    rsx! {
        div { class: "page",
            SectionTitle { title: "Platform console", subtitle: "Plugins, services and developer activity" }
            div { class: "grid-4",
                StatCard { label: "Plugins running", value: format!("{running}/{}", PLUGINS.len()), icon: "extension", tone: "teal", delta: Some("live") }
                StatCard { label: "SDKs published", value: format!("{}", sdks_out), icon: "inventory_2", tone: "indigo", delta: None }
                StatCard { label: "System uptime", value: "99.98%".to_string(), icon: "monitor_heart", tone: "green", delta: Some("30 days") }
                StatCard { label: "Webhooks live", value: format!("{}", hooks_live), icon: "webhook", tone: "amber", delta: None }
            }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { style: "font-weight: 600", "Service health" }
                    Button { variant: "text", icon: "monitor_heart", onclick: |_| {}, "Run diagnostics" }
                }
                table { class: "md-table",
                    thead { tr { th { "Service" } th { "Detail" } th { "Status" } } }
                    tbody {
                        for s in SERVICES.iter() {
                            tr {
                                td { "{s.name}" }
                                td { class: "muted", "{s.detail}" }
                                td { StatusChip { status: s.status.to_string(), label: Some(if s.status == "ok" { "operational".to_string() } else { s.status.to_string() }) } }
                            }
                        }
                    }
                }
            }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { style: "font-weight: 600", "Recent platform activity" }
                }
                table { class: "md-table",
                    thead { tr { th { "Event" } th { "Actor" } th { "When" } } }
                    tbody {
                        for a in ACTIVITY.iter() {
                            tr {
                                td {
                                    span { class: "material-symbols-outlined", style: "font-size: 18px; vertical-align: middle; margin-right: 8px", "{a.icon}" }
                                    "{a.text}"
                                }
                                td { class: "muted", "Platform" }
                                td { class: "muted", "{a.time}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn admin_users(search: &str) -> Element {
    let q = search.trim().to_lowercase();
    rsx! {
        div { class: "page",
            SectionTitle { title: "Users", subtitle: "Manage platform users" }
            div { class: "grid-4",
                StatCard { label: "Total users", value: format!("{}", USERS.len()), icon: "group", tone: "teal", delta: None }
                StatCard { label: "Active", value: format!("{}", USERS.iter().filter(|u| u.status == "active").count()), icon: "check_circle", tone: "green", delta: None }
                StatCard { label: "Suspended", value: format!("{}", USERS.iter().filter(|u| u.status == "suspended").count()), icon: "block", tone: "red", delta: None }
                StatCard { label: "By institution", value: format!("{}", INSTITUTIONS.len()), icon: "account_balance", tone: "indigo", delta: None }
            }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { style: "font-weight: 600", "User management" }
                    Button { variant: "filled", icon: "person_add", onclick: |_| {}, "Add user" }
                }
                table { class: "md-table",
                    thead { tr { th { "Name" } th { "Email" } th { "Role" } th { "Institution" } th { "Status" } } }
                    tbody {
                        for u in USERS.iter().filter(|u| q.is_empty() || u.name.to_lowercase().contains(&q) || u.email.to_lowercase().contains(&q)) {
                            tr {
                                td { "{u.name}" }
                                td { class: "muted", "{u.email}" }
                                td {
                                    span { class: "material-symbols-outlined", style: "font-size: 16px; vertical-align: middle; margin-right: 6px", "{role_icon(u.role)}" }
                                    "{u.role}"
                                }
                                td { class: "muted", "{u.institution}" }
                                td { StatusChip { status: u.status.to_string(), label: Some(u.status.to_string()) } }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn role_icon(role: &str) -> &str {
    for (r, icon) in ROLE_ICONS.iter() {
        if *r == role {
            return icon;
        }
    }
    "person"
}

fn perm_granted(perm: usize, role: usize) -> bool {
    PERM_MATRIX[perm][role]
}

pub fn admin_institutions(search: &str) -> Element {
    let _ = search;
    let total: u32 = INSTITUTIONS.iter().map(|i| i.students).sum();
    let total_c: u32 = INSTITUTIONS.iter().map(|i| i.courses).sum();
    rsx! {
        div { class: "page",
            SectionTitle { title: "Institutions", subtitle: "Manage institutional accounts" }
            div { class: "grid-4",
                StatCard { label: "Institutions", value: format!("{}", INSTITUTIONS.len()), icon: "account_balance", tone: "teal", delta: None }
                StatCard { label: "Total students", value: format!("{total}"), icon: "school", tone: "indigo", delta: None }
                StatCard { label: "Total courses", value: format!("{total_c}"), icon: "menu_book", tone: "green", delta: None }
                StatCard { label: "Enterprise tier", value: format!("{}", INSTITUTIONS.iter().filter(|i| i.tier == "Enterprise").count()), icon: "star", tone: "amber", delta: None }
            }
            div { class: "grid-2",
                for i in INSTITUTIONS.iter() {
                    div { class: "md-card", style: "padding: 20px; display: flex; flex-direction: column; gap: 12px",
                        div { style: "display: flex; align-items: center; justify-content: space-between",
                            h3 { "{i.name}" }
                            span { class: "material-symbols-outlined", "star" }
                        }
                        p { class: "muted", "{i.students} students · {i.courses} courses" }
                        div { class: "tags",
                            span { class: "md-chip", "{i.tier}" }
                        }
                        Button { variant: "tonal", icon: "manage_accounts", onclick: |_| {}, "Manage" }
                    }
                }
            }
        }
    }
}

pub fn admin_audit() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Audit log", subtitle: "Platform activity and administrative actions" }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    span { style: "font-weight: 600", "Audit trail" }
                }
                table { class: "timeline-table",
                    thead { tr { th { "Event" } th { "Actor" } th { "When" } th { "Type" } } }
                    tbody {
                        for a in AUDIT_LOG.iter() {
                            tr {
                                td {
                                    span { class: format!("material-symbols-outlined timeline-icon"), "{a.0}" }
                                    "{a.1}"
                                }
                                td { class: "muted", "{a.2}" }
                                td { class: "muted", "{a.3}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn admin_overview() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Platform admin", subtitle: "Overview of system health and user activity" }
            div { class: "grid-4",
                StatCard { label: "Total users", value: format!("{}", USERS.len()), icon: "group", tone: "teal", delta: None }
                StatCard { label: "Institutions", value: format!("{}", INSTITUTIONS.len()), icon: "account_balance", tone: "indigo", delta: None }
                StatCard { label: "Services running", value: format!("{}", SERVICES.iter().filter(|s| s.status == "ok").count()), icon: "dns", tone: "green", delta: None }
                StatCard { label: "Plugins active", value: format!("{}", PLUGINS.iter().filter(|p| p.status == "running").count()), icon: "extension", tone: "amber", delta: None }
            }
            div { class: "grid-2",
                div { class: "md-card table-card",
                    div { class: "table-toolbar",
                        span { style: "font-weight: 600", "User role distribution" }
                    }
                    div { class: "chart-bar",
                        for (label, val, tone) in ROLE_SPLIT.iter() {
                                                        div { class: format!("bar-group {tone}"),
                                div { class: "bar", style: format!("height: {:.0}%", val) }
                                div { class: "bar-label", "{label} ({val})" }
                            }
                        }
                    }
                }
                div { class: "md-card table-card",
                    div { class: "table-toolbar",
                        span { style: "font-weight: 600", "Recent audit entries" }
                    }
                    div { class: "timeline-list",
                        for a in AUDIT_LOG.iter().take(5) {
                            div { class: "timeline-item",
                                span { class: format!("material-symbols-outlined timeline-icon"), "{a.0}" }
                                div { class: "timeline-content",
                                    div { class: "timeline-text", "{a.1}" }
                                    div { class: "timeline-meta", "{a.2}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
