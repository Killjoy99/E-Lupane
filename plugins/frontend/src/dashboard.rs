//! E-Lupane dashboard — a Material Design 3 (light) e-learning dashboard with
//! four role personas: Student, Lecturer, Developer and Admin.

use dioxus::prelude::*;

use crate::components::*;

// ---------------------------------------------------------------------------
// Roles & personas
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Student,
    Lecturer,
    Developer,
    Admin,
}

impl Role {
    const ALL: [Role; 4] = [Role::Student, Role::Lecturer, Role::Developer, Role::Admin];

    fn label(self) -> &'static str {
        match self {
            Role::Student => "Student",
            Role::Lecturer => "Lecturer",
            Role::Developer => "Developer",
            Role::Admin => "Admin",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Role::Student => "school",
            Role::Lecturer => "co_present",
            Role::Developer => "code_blocks",
            Role::Admin => "admin_panel_settings",
        }
    }
}

struct Persona {
    name: &'static str,
    subtitle: &'static str,
    initials: &'static str,
    tone: &'static str,
}

fn persona(role: Role) -> &'static Persona {
    match role {
        Role::Student => &Persona {
            name: "Thandiwe Ndlovu",
            subtitle: "BSc Computer Science · Year 2",
            initials: "TN",
            tone: "teal",
        },
        Role::Lecturer => &Persona {
            name: "Dr. Nkosana Dlamini",
            subtitle: "Faculty of Computing",
            initials: "ND",
            tone: "indigo",
        },
        Role::Developer => &Persona {
            name: "Lerato Molefe",
            subtitle: "Platform Engineering",
            initials: "LM",
            tone: "green",
        },
        Role::Admin => &Persona {
            name: "Simphiwe Khumalo",
            subtitle: "System Administration",
            initials: "SK",
            tone: "purple",
        },
    }
}

fn greeting(role: Role) -> &'static str {
    match role {
        Role::Student => "Good morning, Thandiwe",
        Role::Lecturer => "Good morning, Dr. Dlamini",
        Role::Developer => "Good morning, Lerato",
        Role::Admin => "Good morning, Simphiwe",
    }
}

// ---------------------------------------------------------------------------
// Navigation
// ---------------------------------------------------------------------------

struct NavItem {
    label: &'static str,
    icon: &'static str,
    view: &'static str,
}

fn nav_items(role: Role) -> &'static [NavItem] {
    match role {
        Role::Student => &[
            NavItem { label: "Dashboard", icon: "space_dashboard", view: "overview" },
            NavItem { label: "My Courses", icon: "menu_book", view: "courses" },
            NavItem { label: "Assignments", icon: "assignment", view: "assignments" },
            NavItem { label: "Grades", icon: "grade", view: "grades" },
            NavItem { label: "Messages", icon: "chat_bubble", view: "messages" },
        ],
        Role::Lecturer => &[
            NavItem { label: "Dashboard", icon: "space_dashboard", view: "overview" },
            NavItem { label: "My Courses", icon: "menu_book", view: "courses" },
            NavItem { label: "Students", icon: "groups", view: "students" },
            NavItem { label: "Grade Book", icon: "fact_check", view: "gradebook" },
            NavItem { label: "Analytics", icon: "monitoring", view: "analytics" },
        ],
        Role::Developer => &[
            NavItem { label: "Dashboard", icon: "space_dashboard", view: "overview" },
            NavItem { label: "Plugins", icon: "extension", view: "plugins" },
            NavItem { label: "SDKs", icon: "inventory_2", view: "sdks" },
            NavItem { label: "System Health", icon: "monitor_heart", view: "system" },
            NavItem { label: "API Docs", icon: "description", view: "docs" },
        ],
        Role::Admin => &[
            NavItem { label: "Dashboard", icon: "space_dashboard", view: "overview" },
            NavItem { label: "Users", icon: "group", view: "users" },
            NavItem { label: "Institutions", icon: "account_balance", view: "institutions" },
            NavItem { label: "Plugins", icon: "extension", view: "plugins" },
            NavItem { label: "Permissions", icon: "lock", view: "permissions" },
        ],
    }
}

// ---------------------------------------------------------------------------
// Mock data
// ---------------------------------------------------------------------------

struct Course {
    code: &'static str,
    name: &'static str,
    instructor: &'static str,
    progress: f64,
    students: u32,
    pending: u32,
    tone: &'static str,
    icon: &'static str,
}

const COURSES: &[Course] = &[
    Course { code: "CS101", name: "Intro to Programming (Python)", instructor: "Dr. N. Dlamini", progress: 72.0, students: 38, pending: 5, tone: "teal", icon: "code" },
    Course { code: "DB201", name: "Database Systems (PostgreSQL)", instructor: "Prof. T. Moyo", progress: 45.0, students: 42, pending: 3, tone: "indigo", icon: "database" },
    Course { code: "WEB302", name: "Full-Stack Web (Rust + Dioxus)", instructor: "Dr. K. Ncube", progress: 88.0, students: 25, pending: 2, tone: "green", icon: "web" },
    Course { code: "MATH150", name: "Linear Algebra", instructor: "Prof. L. Sibanda", progress: 34.0, students: 51, pending: 7, tone: "amber", icon: "functions" },
    Course { code: "AI410", name: "Machine Learning", instructor: "Dr. P. Khumalo", progress: 12.0, students: 18, pending: 0, tone: "purple", icon: "smart_toy" },
];

struct Assignment {
    title: &'static str,
    course: &'static str,
    due: &'static str,
    status: &'static str,
    grade: Option<&'static str>,
}

const ASSIGNMENTS: &[Assignment] = &[
    Assignment { title: "Sorting algorithms — Part 2", course: "CS101", due: "Today, 23:59", status: "pending", grade: None },
    Assignment { title: "Normalisation exercise", course: "DB201", due: "Fri, 17:00", status: "submitted", grade: None },
    Assignment { title: "Dioxus fullstack mini-project", course: "WEB302", due: "Mon, 23:59", status: "graded", grade: Some("87%") },
    Assignment { title: "Matrix multiplication", course: "MATH150", due: "Next Tue, 23:59", status: "pending", grade: None },
];

struct Quiz {
    title: &'static str,
    course: &'static str,
    when: &'static str,
    info: &'static str,
}

const QUIZZES: &[Quiz] = &[
    Quiz { title: "Week 4 quiz · Functions", course: "CS101", when: "Thu 14:00", info: "10 questions · 15 min" },
    Quiz { title: "SQL joins quiz", course: "DB201", when: "Fri 09:00", info: "12 questions · 20 min" },
];

struct Announcement {
    title: &'static str,
    body: &'static str,
    author: &'static str,
    time: &'static str,
}

const ANNOUNCEMENTS: &[Announcement] = &[
    Announcement { title: "Examination timetable published", body: "The May/June exam timetable is now available on the registrar's portal.", author: "Office of the Registrar", time: "2h ago" },
    Announcement { title: "Library extended hours during exams", body: "All campus libraries stay open until 22:00 from next Monday.", author: "Library Services", time: "1d ago" },
    Announcement { title: "Plugin SDK workshop", body: "Hands-on session on building E-Lupane plugins with the Rust & Python SDKs.", author: "Platform Team", time: "3d ago" },
];

struct StudentRow {
    name: &'static str,
    course: &'static str,
    grade: &'static str,
    status: &'static str,
}

const ROSTER: &[StudentRow] = &[
    StudentRow { name: "Aisha Patel", course: "CS101", grade: "93%", status: "on-track" },
    StudentRow { name: "Sipho Ndlovu", course: "CS101", grade: "82%", status: "on-track" },
    StudentRow { name: "Naledi Mokoena", course: "CS101", grade: "74%", status: "on-track" },
    StudentRow { name: "Tendai Moyo", course: "CS101", grade: "68%", status: "on-track" },
    StudentRow { name: "Kgotso Molefe", course: "CS101", grade: "61%", status: "at-risk" },
    StudentRow { name: "Lerato Dube", course: "CS101", grade: "55%", status: "at-risk" },
];

struct Submission {
    name: &'static str,
    item: &'static str,
    course: &'static str,
    time: &'static str,
    state: &'static str,
}

const SUBMISSIONS: &[Submission] = &[
    Submission { name: "Aisha Patel", item: "Sorting algorithms — Part 2", course: "CS101", time: "12 min ago", state: "new" },
    Submission { name: "Tendai Moyo", item: "Normalisation exercise", course: "DB201", time: "1h ago", state: "new" },
    Submission { name: "Kgotso Molefe", item: "Sorting algorithms — Part 2", course: "CS101", time: "2h ago", state: "flagged" },
    Submission { name: "Naledi Mokoena", item: "Dioxus mini-project", course: "WEB302", time: "Yesterday", state: "graded" },
];

struct Deadline {
    title: &'static str,
    course: &'static str,
    when: &'static str,
    kind: &'static str,
}

const DEADLINES: &[Deadline] = &[
    Deadline { title: "Grade sorting algorithms — Part 2", course: "CS101", when: "Today, 23:59", kind: "grading" },
    Deadline { title: "Publish week 5 lab brief", course: "CS101", when: "Thu 12:00", kind: "coursework" },
    Deadline { title: "Finalise DB201 marks", course: "DB201", when: "Fri 17:00", kind: "grading" },
];


struct PluginRow {
    name: &'static str,
    lang: &'static str,
    version: &'static str,
    status: &'static str,
    signed: bool,
    port: u16,
}

const PLUGINS: &[PluginRow] = &[
    PluginRow { name: "institution_management", lang: "Rust", version: "0.1.0", status: "running", signed: true, port: 9101 },
    PluginRow { name: "course_management", lang: "Rust", version: "0.1.0", status: "running", signed: true, port: 9102 },
    PluginRow { name: "assignments", lang: "Rust", version: "0.1.0", status: "running", signed: true, port: 9103 },
    PluginRow { name: "quizzes", lang: "Python", version: "0.1.0", status: "running", signed: true, port: 9104 },
    PluginRow { name: "analytics", lang: "Rust", version: "0.2.1", status: "degraded", signed: true, port: 9105 },
    PluginRow { name: "notifications", lang: "Python", version: "0.1.0", status: "running", signed: true, port: 9106 },
];

struct Service {
    name: &'static str,
    status: &'static str,
    detail: &'static str,
}

const SERVICES: &[Service] = &[
    Service { name: "Gateway", status: "ok", detail: "actix-web · port 8080" },
    Service { name: "PostgreSQL", status: "ok", detail: "elupane · 12 connections" },
    Service { name: "Redis", status: "ok", detail: "event bus & cache" },
    Service { name: "Event bus", status: "ok", detail: "0 failed publishes" },
    Service { name: "Media service", status: "ok", detail: "S3 storage · 84% free" },
    Service { name: "Search service", status: "warn", detail: "index lag · 2m behind" },
];

struct Sdk {
    lang: &'static str,
    version: &'static str,
    status: &'static str,
    note: &'static str,
}

const SDKS: &[Sdk] = &[
    Sdk { lang: "Rust", version: "0.1.0", status: "available", note: "Native, first-class support" },
    Sdk { lang: "Python", version: "0.1.0", status: "available", note: "Built with maturin" },
    Sdk { lang: "WebAssembly", version: "0.1.0-beta", status: "review", note: "Sandboxed WASM plugins" },
    Sdk { lang: "PHP", version: "—", status: "pending", note: "Structured shared objects" },
    Sdk { lang: "C#", version: "—", status: "pending", note: "On the roadmap" },
];

struct UserRow {
    name: &'static str,
    email: &'static str,
    role: &'static str,
    institution: &'static str,
    status: &'static str,
}

const USERS: &[UserRow] = &[
    UserRow { name: "Aisha Patel", email: "aisha.patel@lupane.ac.zw", role: "Student", institution: "Lupane State University", status: "active" },
    UserRow { name: "Dr. Nkosana Dlamini", email: "n.dlamini@lupane.ac.zw", role: "Lecturer", institution: "Lupane State University", status: "active" },
    UserRow { name: "Lerato Molefe", email: "lerato@elupane.dev", role: "Developer", institution: "Platform", status: "active" },
    UserRow { name: "Kgotso Molefe", email: "kgotso.m@bit.ac.zw", role: "Student", institution: "Bulawayo Institute of Tech", status: "active" },
    UserRow { name: "Prof. L. Sibanda", email: "l.sibanda@hsc.ac.zw", role: "Lecturer", institution: "Harare School of Computing", status: "suspended" },
    UserRow { name: "Simphiwe Khumalo", email: "skhumalo@elupane.dev", role: "Admin", institution: "Platform", status: "active" },
];

struct Institution {
    name: &'static str,
    students: u32,
    courses: u32,
    tier: &'static str,
}

const INSTITUTIONS: &[Institution] = &[
    Institution { name: "Lupane State University", students: 612, courses: 34, tier: "Enterprise" },
    Institution { name: "Bulawayo Institute of Technology", students: 284, courses: 19, tier: "Standard" },
    Institution { name: "Harare School of Computing", students: 198, courses: 12, tier: "Standard" },
    Institution { name: "Midlands Innovation College", students: 154, courses: 9, tier: "Starter" },
];

struct Approval {
    plugin: &'static str,
    lang: &'static str,
    requester: &'static str,
    key: &'static str,
}

const APPROVALS: &[Approval] = &[
    Approval { plugin: "analytics v0.3.0", lang: "Rust", requester: "L. Molefe", key: "lupane-ops-7f3a" },
    Approval { plugin: "attendance-tracker", lang: "Python", requester: "M. Ncube", key: "lupane-dev-91b2" },
];

struct Activity {
    icon: &'static str,
    text: &'static str,
    time: &'static str,
    tone: &'static str,
}

const ACTIVITY: &[Activity] = &[
    Activity { icon: "verified", text: "Plugin `analytics v0.3.0` signed & approved", time: "4h ago", tone: "teal" },
    Activity { icon: "upload_file", text: "Bulk import — 214 students added", time: "1d ago", tone: "indigo" },
    Activity { icon: "domain", text: "Harare School of Computing onboarded", time: "2d ago", tone: "green" },
    Activity { icon: "key", text: "New signing key issued to developer M. Ncube", time: "3d ago", tone: "amber" },
];

const ROLE_SPLIT: &[(&str, u32, &str)] = &[
    ("Students", 1015, "teal"),
    ("Lecturers", 126, "indigo"),
    ("Developers", 22, "green"),
    ("Admins", 5, "purple"),
];


// ---------------------------------------------------------------------------
// App shell
// ---------------------------------------------------------------------------

/// Top-level dashboard: role-aware app shell + content routing.
pub fn Dashboard() -> Element {
    let role = use_signal(|| Role::Student);
    let active_view = use_signal(|| "overview");

    rsx! {
        div { class: "md-app",
            Sidebar { role: role, active_view: active_view }
            div { class: "md-main",
                TopBar { role: role, active_view: active_view }
                main { class: "md-content",
                    {render_page(role(), active_view())}
                }
            }
        }
    }
}

#[component]
fn Sidebar(role: Signal<Role>, active_view: Signal<&'static str>) -> Element {
    let current = role();
    rsx! {
        aside { class: "md-drawer",
            div { class: "md-brand",
                div { class: "md-brand-logo",
                    span { class: "material-symbols-outlined", "rocket_launch" }
                }
                div {
                    div { class: "md-brand-name", "E-Lupane" }
                    div { class: "md-brand-sub", "Learning OS" }
                }
            }

            div { class: "md-nav",
                p { class: "md-nav-title", "ROLE" }
                for r in Role::ALL {
                    button {
                        class: format!("md-role-btn {}", if r == current { "active" } else { "" }),
                        onclick: move |_| {
                            role.set(r);
                            active_view.set("overview");
                        },
                        span { class: "material-symbols-outlined", "{r.icon()}" }
                        span { "{r.label()}" }
                        if r == current {
                            span { class: "md-role-check material-symbols-outlined", "check_circle" }
                        }
                    }
                }

                p { class: "md-nav-title", "MENU" }
                for item in nav_items(current) {
                    button {
                        class: format!(
                            "md-nav-item {}",
                            if active_view() == item.view { "active" } else { "" }
                        ),
                        onclick: move |_| active_view.set(item.view),
                        span { class: "material-symbols-outlined", "{item.icon}" }
                        span { "{item.label}" }
                    }
                }
            }

            div { class: "md-drawer-user",
                Avatar { initials: persona(current).initials, tone: persona(current).tone }
                div {
                    div { class: "md-drawer-user-name", "{persona(current).name}" }
                    div { class: "md-drawer-user-sub", "{current.label()}" }
                }
            }
        }
    }
}

#[component]
fn TopBar(role: Signal<Role>, active_view: Signal<&'static str>) -> Element {
    let current = role();
    rsx! {
        header { class: "md-topbar",
            div { class: "md-search",
                span { class: "material-symbols-outlined", "search" }
                input { placeholder: "Search courses, people, plugins…" }
            }

            div { class: "md-topbar-spacer" }

            div { class: "md-topbar-role",
                span { class: "material-symbols-outlined", "{current.icon()}" }
                span { "{current.label()} view" }
            }

            button { class: "md-icon-btn",
                span { class: "material-symbols-outlined", "notifications" }
                span { class: "md-badge", "3" }
            }

            div { class: "md-topbar-user",
                Avatar { initials: persona(current).initials, tone: persona(current).tone }
                div { class: "md-topbar-user-meta",
                    div { class: "md-topbar-user-name", "{persona(current).name}" }
                    div { class: "md-topbar-user-sub", "{persona(current).subtitle}" }
                }
            }
        }
    }
}


// ---------------------------------------------------------------------------
// Student pages
// ---------------------------------------------------------------------------

fn pct(v: f64) -> String {
    format!("{v:.0}%")
}

fn course_card(c: &Course) -> Element {
    rsx! {
        div { class: "md-card course-card",
            div { class: "course-card-head",
                span { class: format!("course-icon course-icon-{}", c.tone),
                    span { class: "material-symbols-outlined", "{c.icon}" }
                }
                span { class: "course-code", "{c.code}" }
            }
            h3 { "{c.name}" }
            div { class: "course-meta", "{c.instructor}" }
            div { class: "course-progress-row",
                ProgressBar { value: c.progress, tone: c.tone }
                span { class: "course-progress-pct", "{pct(c.progress)}" }
            }
            div { class: "course-foot",
                span { class: "course-students", "{c.students} students" }
                Button { variant: "tonal", icon: "play_arrow", onclick: |_| {}, "Resume" }
            }
        }
    }
}

fn student_overview() -> Element {
    rsx! {
        div { class: "page",
            div { class: "hero",
                div { class: "hero-text",
                    h1 { "{greeting(Role::Student)} 👋" }
                    p { "You have 3 assignments due this week and 1 quiz tomorrow. Keep the momentum going!" }
                    div { class: "hero-actions",
                        Button { icon: "play_circle", onclick: |_| {}, "Continue learning" }
                        Button { variant: "tonal", icon: "event", onclick: |_| {}, "View timetable" }
                    }
                }
                div { class: "hero-art",
                    span { class: "material-symbols-outlined", "school" }
                }
            }

            div { class: "grid-4",
                StatCard { label: "Courses in progress", value: "6".to_string(), icon: "menu_book", tone: "teal", delta: "+1 this term" }
                StatCard { label: "Average grade", value: "78%".to_string(), icon: "trending_up", tone: "indigo", delta: "+3.2%" }
                StatCard { label: "Pending assignments", value: "3".to_string(), icon: "assignment_late", tone: "amber", delta: "2 due today" }
                StatCard { label: "Certificates", value: "2".to_string(), icon: "workspace_premium", tone: "green", delta: "1 in progress" }
            }

            SectionTitle {
                title: "Continue learning",
                subtitle: "Pick up where you left off",
                action: rsx! { Button { variant: "text", icon: "arrow_forward", onclick: |_| {}, "View all" } }
            }
            div { class: "grid-3",
                for c in COURSES.iter().take(3) {
                    {course_card(c)}
                }
            }

            div { class: "grid-2",
                Card {
                    div { class: "card-title-row",
                        h3 { "Upcoming deadlines" }
                        Chip { label: "This week".to_string(), selected: true }
                    }
                    div { class: "list",
                        for a in ASSIGNMENTS.iter().take(2) {
                            div { class: "list-row",
                                span { class: "icon-box icon-box-warn",
                                    span { class: "material-symbols-outlined", "assignment" }
                                }
                                div { class: "list-row-body",
                                    div { class: "list-row-title", "{a.title}" }
                                    div { class: "list-row-sub", "{a.course} · due {a.due}" }
                                }
                                StatusChip { status: a.status }
                            }
                        }
                        for q in QUIZZES.iter() {
                            div { class: "list-row",
                                span { class: "icon-box icon-box-indigo",
                                    span { class: "material-symbols-outlined", "quiz" }
                                }
                                div { class: "list-row-body",
                                    div { class: "list-row-title", "{q.title}" }
                                    div { class: "list-row-sub", "{q.course} · {q.when} · {q.info}" }
                                }
                                Button { variant: "tonal", icon: "bolt", onclick: |_| {}, "Prepare" }
                            }
                        }
                    }
                }
                Card {
                    div { class: "card-title-row",
                        h3 { "Announcements" }
                        Button { variant: "text", icon: "campaign", onclick: |_| {}, "All" }
                    }
                    div { class: "list",
                        for a in ANNOUNCEMENTS.iter() {
                            div { class: "list-row",
                                span { class: "icon-box icon-box-teal",
                                    span { class: "material-symbols-outlined", "campaign" }
                                }
                                div { class: "list-row-body",
                                    div { class: "list-row-title", "{a.title}" }
                                    div { class: "list-row-sub", "{a.author} · {a.time}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


fn student_courses() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "My courses", subtitle: "6 courses · 2 completed this term" }
            div { class: "grid-3",
                for c in COURSES.iter() {
                    {course_card(c)}
                }
            }
        }
    }
}

fn student_assignments() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Assignments",
                subtitle: "3 pending · 1 submitted · 1 graded",
                action: rsx! { Chip { label: "Sort by: due date".to_string(), icon: "swap_vert" } }
            }
            div { class: "md-card table-wrap",
                table { class: "md-table",
                    thead { tr {
                        th { "Assignment" }
                        th { "Course" }
                        th { "Due" }
                        th { "Status" }
                        th { "Grade" }
                    } }
                    tbody {
                        for a in ASSIGNMENTS.iter() {
                            tr {
                                td { class: "td-strong", "{a.title}" }
                                td { "{a.course}" }
                                td { "{a.due}" }
                                td { StatusChip { status: a.status } }
                                td {
                                    if let Some(g) = a.grade {
                                        span { class: "grade-pill", "{g}" }
                                    } else {
                                        span { class: "muted", "—" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn student_grades() -> Element {
    const GRADES: &[(&str, &str, &str, &str)] = &[
        ("CS101", "Week 3 quiz", "A", "92%"),
        ("CS101", "Sorting algorithms — Part 1", "A−", "88%"),
        ("WEB302", "Dioxus mini-project", "B+", "87%"),
        ("DB201", "Normalisation exercise", "B", "81%"),
        ("MATH150", "Problem set 4", "C+", "73%"),
    ];
    rsx! {
        div { class: "page",
            SectionTitle { title: "Grades", subtitle: "Cumulative average: 78% (B)" }
            div { class: "md-card table-wrap",
                table { class: "md-table",
                    thead { tr {
                        th { "Course" }
                        th { "Assessment" }
                        th { "Grade" }
                        th { "Score" }
                    } }
                    tbody {
                        for &(code, item, grade, score) in GRADES {
                            tr {
                                td { class: "td-strong", "{code}" }
                                td { "{item}" }
                                td { span { class: "grade-pill", "{grade}" } }
                                td { "{score}" }
                            }
                        }
                    }
                }
            }
        }
    }
}


// ---------------------------------------------------------------------------
// Lecturer pages
// ---------------------------------------------------------------------------

fn lecturer_course_card(c: &Course) -> Element {
    rsx! {
        div { class: "md-card course-card",
            div { class: "course-card-head",
                span { class: format!("course-icon course-icon-{}", c.tone),
                    span { class: "material-symbols-outlined", "{c.icon}" }
                }
                span { class: "course-code", "{c.code}" }
            }
            h3 { "{c.name}" }
            div { class: "course-meta", "{c.students} students enrolled" }
            div { class: "course-progress-row",
                ProgressBar { value: c.progress, tone: c.tone }
                span { class: "course-progress-pct", "{pct(c.progress)}" }
            }
            div { class: "course-foot",
                span { class: "course-students",
                    if c.pending > 0 {
                        span { class: "badge-pill", "{c.pending} to grade" }
                    } else {
                        span { class: "muted", "All graded" }
                    }
                }
                Button { variant: "tonal", icon: "tune", onclick: |_| {}, "Manage" }
            }
        }
    }
}

fn lecturer_overview() -> Element {
    rsx! {
        div { class: "page",
            div { class: "hero",
                div { class: "hero-text",
                    h1 { "{greeting(Role::Lecturer)} 👋" }
                    p { "4 active courses · 126 students · 12 submissions are waiting to be graded." }
                    div { class: "hero-actions",
                        Button { icon: "fact_check", onclick: |_| {}, "Open grade book" }
                        Button { variant: "tonal", icon: "add_circle", onclick: |_| {}, "New course" }
                    }
                }
                div { class: "hero-art",
                    span { class: "material-symbols-outlined", "co_present" }
                }
            }

            div { class: "grid-4",
                StatCard { label: "Active courses", value: "4".to_string(), icon: "menu_book", tone: "teal", delta: "+1 this term" }
                StatCard { label: "Total students", value: "126".to_string(), icon: "groups", tone: "indigo", delta: "+14" }
                StatCard { label: "Needs grading", value: "12".to_string(), icon: "rate_review", tone: "amber", delta: "5 due today" }
                StatCard { label: "Avg. rating", value: "4.7".to_string(), icon: "star", tone: "green", delta: "★ from 214 reviews" }
            }

            SectionTitle {
                title: "My courses",
                subtitle: "3 actively in progress",
                action: rsx! { Button { variant: "text", icon: "arrow_forward", onclick: |_| {}, "All courses" } }
            }
            div { class: "grid-3",
                for c in COURSES.iter().take(3) {
                    {lecturer_course_card(c)}
                }
            }

            div { class: "grid-2",
                Card {
                    div { class: "card-title-row",
                        h3 { "Recent submissions" }
                        Button { variant: "text", icon: "fact_check", onclick: |_| {}, "Grade book" }
                    }
                    div { class: "list",
                        for s in SUBMISSIONS.iter() {
                            div { class: "list-row",
                                span { class: "icon-box icon-box-teal",
                                    span { class: "material-symbols-outlined", "description" }
                                }
                                div { class: "list-row-body",
                                    div { class: "list-row-title", "{s.name} · {s.item}" }
                                    div { class: "list-row-sub", "{s.course} · {s.time}" }
                                }
                                StatusChip { status: s.state }
                            }
                        }
                    }
                }
                Card {
                    div { class: "card-title-row",
                        h3 { "Upcoming deadlines" }
                        Chip { label: "This week".to_string(), selected: true }
                    }
                    div { class: "list",
                        for d in DEADLINES.iter() {
                            div { class: "list-row",
                                span { class: "icon-box icon-box-amber",
                                    span { class: "material-symbols-outlined", "schedule" }
                                }
                                div { class: "list-row-body",
                                    div { class: "list-row-title", "{d.title}" }
                                    div { class: "list-row-sub", "{d.course} · {d.when}" }
                                }
                                Chip { label: d.kind.to_string() }
                            }
                        }
                    }
                }
            }
        }
    }
}


fn lecturer_courses() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle {
                title: "My courses",
                subtitle: "4 courses · 3 in progress · 1 archived",
                action: rsx! { Button { icon: "add_circle", onclick: |_| {}, "New course" } }
            }
            div { class: "grid-3",
                for c in COURSES.iter() {
                    {lecturer_course_card(c)}
                }
            }
        }
    }
}

fn lecturer_students() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Students",
                subtitle: "CS101 · 38 enrolled · 2 at risk",
                action: rsx! { Chip { label: "Filter: CS101".to_string(), icon: "filter_list", selected: true } }
            }
            div { class: "md-card table-wrap",
                table { class: "md-table",
                    thead { tr {
                        th { "Student" }
                        th { "Course" }
                        th { "Current grade" }
                        th { "Status" }
                    } }
                    tbody {
                        for s in ROSTER.iter() {
                            tr {
                                td { class: "td-strong", "{s.name}" }
                                td { "{s.course}" }
                                td { "{s.grade}" }
                                td { StatusChip { status: s.status } }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn lecturer_gradebook() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Grade book",
                subtitle: "12 items waiting · 4 graded recently",
                action: rsx! { Button { icon: "download", onclick: |_| {}, "Export" } }
            }
            div { class: "md-card table-wrap",
                table { class: "md-table",
                    thead { tr {
                        th { "Student" }
                        th { "Assignment" }
                        th { "Course" }
                        th { "Submitted" }
                        th { "Status" }
                        th { "" }
                    } }
                    tbody {
                        for s in SUBMISSIONS.iter() {
                            tr {
                                td { class: "td-strong", "{s.name}" }
                                td { "{s.item}" }
                                td { "{s.course}" }
                                td { "{s.time}" }
                                td { StatusChip { status: s.state } }
                                td {
                                    if s.state != "graded" {
                                        Button { variant: "tonal", icon: "rate_review", onclick: |_| {}, "Grade" }
                                    } else {
                                        span { class: "muted", "Done" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


// ---------------------------------------------------------------------------
// Developer pages
// ---------------------------------------------------------------------------

fn DeveloperOverview() -> Element {
    let mut health = use_action(|| async move { crate::server_functions::health().await });

    rsx! {
        div { class: "page",
            div { class: "hero",
                div { class: "hero-text",
                    h1 { "{greeting(Role::Developer)} 👋" }
                    p { "6 plugins running across 3 languages. PostgreSQL, Redis and the event bus are healthy." }
                    div { class: "hero-actions",
                        Button { icon: "monitor_heart", onclick: move |_| health.call(), "Run diagnostics" }
                        Button { variant: "tonal", icon: "add_circle", onclick: |_| {}, "New plugin" }
                    }
                }
                div { class: "hero-art",
                    span { class: "material-symbols-outlined", "code_blocks" }
                }
            }

            div { class: "grid-4",
                StatCard { label: "Plugins online", value: "6/6".to_string(), icon: "extension", tone: "teal", delta: "1 degraded" }
                StatCard { label: "Supported languages", value: "3".to_string(), icon: "translate", tone: "indigo", delta: "2 more planned" }
                StatCard { label: "API uptime", value: "99.98%".to_string(), icon: "monitor_heart", tone: "green", delta: "30-day window" }
                StatCard { label: "Avg. latency", value: "12 ms".to_string(), icon: "bolt", tone: "amber", delta: "-2 ms" }
            }

            SectionTitle {
                title: "Plugin registry",
                subtitle: "All plugins run isolated over gRPC",
                action: rsx! { Button { variant: "text", icon: "arrow_forward", onclick: |_| {}, "All plugins" } }
            }
            div { class: "md-card table-wrap",
                table { class: "md-table",
                    thead { tr {
                        th { "Plugin" }
                        th { "Language" }
                        th { "Version" }
                        th { "Port" }
                        th { "Signature" }
                        th { "Status" }
                    } }
                    tbody {
                        for p in PLUGINS.iter() {
                            tr {
                                td { class: "td-strong mono", "{p.name}" }
                                td { "{p.lang}" }
                                td { "v{p.version}" }
                                td { class: "mono", "{p.port}" }
                                td {
                                    if p.signed {
                                        StatusChip { status: "signed" }
                                    } else {
                                        StatusChip { status: "warn", label: Some("unsigned") }
                                    }
                                }
                                td { StatusChip { status: p.status } }
                            }
                        }
                    }
                }
            }

            div { class: "grid-2",
                Card {
                    div { class: "card-title-row",
                        h3 { "System services" }
                        if let Some(res) = health.value() {
                            match res {
                                Ok(h) => {
                                    let connected = h().database == "connected";
                                    let status = if connected { "ok" } else { "warn" };
                                    let label = if connected { "DB connected" } else { "DB unavailable" };
                                    rsx! {
                                        StatusChip {
                                            status: status,
                                            label: Some(label),
                                        }
                                    }
                                },
                                Err(_) => rsx! { StatusChip { status: "danger", label: Some("Check failed") } },
                            }
                        } else {
                            Chip { label: "Idle".to_string(), icon: "sensors" }
                        }
                    }
                    div { class: "list",
                        for s in SERVICES.iter() {
                            div { class: "list-row",
                                span { class: "icon-box icon-box-teal",
                                    span { class: "material-symbols-outlined", "dns" }
                                }
                                div { class: "list-row-body",
                                    div { class: "list-row-title", "{s.name}" }
                                    div { class: "list-row-sub", "{s.detail}" }
                                }
                                StatusChip { status: s.status }
                            }
                        }
                    }
                }
                Card {
                    div { class: "card-title-row",
                        h3 { "Latest activity" }
                        Chip { label: "Past 7 days".to_string(), icon: "history" }
                    }
                    div { class: "list",
                        for a in ACTIVITY.iter() {
                            div { class: "list-row",
                                span { class: format!("icon-box icon-box-{}", a.tone),
                                    span { class: "material-symbols-outlined", "{a.icon}" }
                                }
                                div { class: "list-row-body",
                                    div { class: "list-row-title", "{a.text}" }
                                    div { class: "list-row-sub", "{a.time}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


fn developer_plugins() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Plugins",
                subtitle: "6 registered · 1 pending update · all signatures valid",
                action: rsx! { Button { icon: "add_circle", onclick: |_| {}, "Register plugin" } }
            }
            div { class: "md-card table-wrap",
                table { class: "md-table",
                    thead { tr {
                        th { "Plugin" }
                        th { "Language" }
                        th { "Version" }
                        th { "Port" }
                        th { "Signature" }
                        th { "Status" }
                        th { "" }
                    } }
                    tbody {
                        for p in PLUGINS.iter() {
                            tr {
                                td { class: "td-strong mono", "{p.name}" }
                                td { "{p.lang}" }
                                td { "v{p.version}" }
                                td { class: "mono", "{p.port}" }
                                td {
                                    if p.signed {
                                        StatusChip { status: "signed" }
                                    } else {
                                        StatusChip { status: "warn", label: Some("unsigned") }
                                    }
                                }
                                td { StatusChip { status: p.status } }
                                td {
                                    Button { variant: "tonal", icon: "settings", onclick: |_| {}, "Manage" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn developer_sdks() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle {
                title: "SDKs",
                subtitle: "Plugin languages — each developer gets a signing key",
                action: rsx! { Button { icon: "download", onclick: |_| {}, "Download SDK" } }
            }
            div { class: "grid-3",
                for s in SDKS.iter() {
                    div { class: "md-card sdk-card",
                        span { class: format!("sdk-badge sdk-badge-{}", if s.status == "available" { "ok" } else { "warn" }),
                            span { class: "material-symbols-outlined",
                                if s.lang == "Rust" { "rust" }
                                else if s.lang == "Python" { "python" }
                                else if s.lang == "WebAssembly" { "web_asset" }
                                else { "extension" }
                            }
                        }
                        h3 { "{s.lang}" }
                        div { class: "sdk-version", "v{s.version}" }
                        div { class: "sdk-note", "{s.note}" }
                        div { class: "sdk-foot",
                            StatusChip { status: s.status }
                            Button { variant: "text", icon: "arrow_forward", onclick: |_| {}, "Docs" }
                        }
                    }
                }
            }
        }
    }
}

fn developer_system() -> Element {
    const RESOURCES: &[(&str, f64, &str, &str)] = &[
        ("CPU", 34.0, "teal", "4 vCPU · 1.2 GHz avg"),
        ("Memory", 52.0, "indigo", "5.2 GB / 10 GB"),
        ("Storage", 61.0, "amber", "1.2 TB / 2 TB"),
        ("Network", 28.0, "green", "1.2 Gbps peak"),
    ];
    rsx! {
        div { class: "page",
            SectionTitle { title: "System health", subtitle: "Live view of services powering E-Lupane" }
            div { class: "grid-2",
                Card {
                    div { class: "card-title-row",
                        h3 { "Services" }
                        Chip { label: "Live".to_string(), icon: "sensors", selected: true }
                    }
                    div { class: "list",
                        for s in SERVICES.iter() {
                            div { class: "list-row",
                                span { class: "icon-box icon-box-teal",
                                    span { class: "material-symbols-outlined", "dns" }
                                }
                                div { class: "list-row-body",
                                    div { class: "list-row-title", "{s.name}" }
                                    div { class: "list-row-sub", "{s.detail}" }
                                }
                                StatusChip { status: s.status }
                            }
                        }
                    }
                }
                Card {
                    div { class: "card-title-row",
                        h3 { "Resource usage" }
                        Chip { label: "Node 01".to_string(), icon: "memory" }
                    }
                    div { class: "resource-list",
                        for &(name, val, tone, detail) in RESOURCES {
                            div { class: "resource-row",
                                div { class: "resource-row-head",
                                    span { "{name}" }
                                    span { "{pct(val)}" }
                                }
                                ProgressBar { value: val, tone: tone }
                                div { class: "resource-detail", "{detail}" }
                            }
                        }
                    }
                }
            }
        }
    }
}


// ---------------------------------------------------------------------------
// Admin pages
// ---------------------------------------------------------------------------

fn admin_overview() -> Element {
    rsx! {
        div { class: "page",
            div { class: "hero",
                div { class: "hero-text",
                    h1 { "{greeting(Role::Admin)} 👋" }
                    p { "1,248 users across 4 institutions. 2 plugin approvals are waiting for your review." }
                    div { class: "hero-actions",
                        Button { icon: "person_add", onclick: |_| {}, "Invite user" }
                        Button { variant: "tonal", icon: "verified", onclick: |_| {}, "Review plugins" }
                    }
                }
                div { class: "hero-art",
                    span { class: "material-symbols-outlined", "admin_panel_settings" }
                }
            }

            div { class: "grid-4",
                StatCard { label: "Total users", value: "1,248".to_string(), icon: "group", tone: "teal", delta: "+214 this month" }
                StatCard { label: "Institutions", value: "4".to_string(), icon: "account_balance", tone: "indigo", delta: "+1 onboarding" }
                StatCard { label: "Active plugins", value: "6".to_string(), icon: "extension", tone: "green", delta: "1 degraded" }
                StatCard { label: "Signing keys", value: "9".to_string(), icon: "key", tone: "amber", delta: "2 pending" }
            }

            div { class: "grid-2",
                Card {
                    div { class: "card-title-row",
                        h3 { "Users by role" }
                        Chip { label: "All institutions".to_string(), icon: "filter_list" }
                    }
                    div { class: "resource-list",
                        for &(name, val, tone) in ROLE_SPLIT {
                            div { class: "resource-row",
                                div { class: "resource-row-head",
                                    span { "{name}" }
                                    span { "{val} · {pct(val as f64 / 1168.0 * 100.0)}" }
                                }
                                ProgressBar { value: val as f64 / 1168.0 * 100.0, tone: tone }
                            }
                        }
                    }
                }
                Card {
                    div { class: "card-title-row",
                        h3 { "Pending plugin approvals" }
                        Chip { label: "2 awaiting".to_string(), icon: "verified", selected: true }
                    }
                    div { class: "list",
                        for a in APPROVALS.iter() {
                            div { class: "list-row approval-row",
                                span { class: "icon-box icon-box-warn",
                                    span { class: "material-symbols-outlined", "extension" }
                                }
                                div { class: "list-row-body",
                                    div { class: "list-row-title", "{a.plugin}" }
                                    div { class: "list-row-sub mono", "{a.requester} · {a.key}" }
                                }
                                div { class: "approval-actions",
                                    Button { icon: "check", onclick: |_| {}, "Approve" }
                                    Button { variant: "text", icon: "close", onclick: |_| {}, "Reject" }
                                }
                            }
                        }
                    }
                }
            }

            SectionTitle { title: "Recent activity", subtitle: "System-wide audit trail" }
            div { class: "md-card",
                div { class: "list",
                    for a in ACTIVITY.iter() {
                        div { class: "list-row",
                            span { class: format!("icon-box icon-box-{}", a.tone),
                                span { class: "material-symbols-outlined", "{a.icon}" }
                            }
                            div { class: "list-row-body",
                                div { class: "list-row-title", "{a.text}" }
                                div { class: "list-row-sub", "{a.time}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn admin_users() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Users",
                subtitle: "1,248 accounts · 2,039 with SSO",
                action: rsx! { Button { icon: "person_add", onclick: |_| {}, "Invite user" } }
            }
            div { class: "md-card table-wrap",
                table { class: "md-table",
                    thead { tr {
                        th { "Name" }
                        th { "Email" }
                        th { "Role" }
                        th { "Institution" }
                        th { "Status" }
                    } }
                    tbody {
                        for u in USERS.iter() {
                            tr {
                                td { class: "td-strong", "{u.name}" }
                                td { class: "mono", "{u.email}" }
                                td { "{u.role}" }
                                td { "{u.institution}" }
                                td { StatusChip { status: u.status } }
                            }
                        }
                    }
                }
            }
        }
    }
}


fn admin_institutions() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Institutions",
                subtitle: "4 active tenants",
                action: rsx! { Button { icon: "add_business", onclick: |_| {}, "Onboard institution" } }
            }
            div { class: "md-card table-wrap",
                table { class: "md-table",
                    thead { tr {
                        th { "Institution" }
                        th { "Students" }
                        th { "Courses" }
                        th { "Tier" }
                        th { "" }
                    } }
                    tbody {
                        for i in INSTITUTIONS.iter() {
                            tr {
                                td { class: "td-strong", "{i.name}" }
                                td { "{i.students}" }
                                td { "{i.courses}" }
                                td { Chip { label: i.tier.to_string() } }
                                td { Button { variant: "tonal", icon: "settings", onclick: |_| {}, "Manage" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn admin_plugins() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Plugin approvals",
                subtitle: "Signing keys are issued per developer and verified before integration",
                action: rsx! { Button { icon: "key", onclick: |_| {}, "Issue signing key" } }
            }
            div { class: "md-card",
                div { class: "list",
                    for a in APPROVALS.iter() {
                        div { class: "list-row approval-row",
                            span { class: "icon-box icon-box-warn",
                                span { class: "material-symbols-outlined", "extension" }
                            }
                            div { class: "list-row-body",
                                div { class: "list-row-title", "{a.plugin}" }
                                div { class: "list-row-sub", "{a.requester} · {a.lang} · key {a.key}" }
                            }
                            div { class: "approval-actions",
                                Button { icon: "check", onclick: |_| {}, "Approve" }
                                Button { variant: "outlined", icon: "close", onclick: |_| {}, "Reject" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Content router
// ---------------------------------------------------------------------------

fn render_page(role: Role, view: &'static str) -> Element {
    match role {
        Role::Student => match view {
            "courses" => rsx! { {student_courses()} },
            "assignments" => rsx! { {student_assignments()} },
            "grades" => rsx! { {student_grades()} },
            "messages" => rsx! { Placeholder { title: "Messages", desc: "Conversations with lecturers and classmates will appear here.", icon: "chat_bubble" } },
            _ => rsx! { {student_overview()} },
        },
        Role::Lecturer => match view {
            "courses" => rsx! { {lecturer_courses()} },
            "students" => rsx! { {lecturer_students()} },
            "gradebook" => rsx! { {lecturer_gradebook()} },
            "analytics" => rsx! { Placeholder { title: "Analytics", desc: "Course analytics and engagement reports are being prepared.", icon: "monitoring" } },
            _ => rsx! { {lecturer_overview()} },
        },
        Role::Developer => match view {
            "plugins" => rsx! { {developer_plugins()} },
            "sdks" => rsx! { {developer_sdks()} },
            "system" => rsx! { {developer_system()} },
            "docs" => rsx! { Placeholder { title: "API docs", desc: "Interactive API reference for the gateway and plugins.", icon: "description" } },
            _ => rsx! { {DeveloperOverview()} },
        },
        Role::Admin => match view {
            "users" => rsx! { {admin_users()} },
            "institutions" => rsx! { {admin_institutions()} },
            "plugins" => rsx! { {admin_plugins()} },
            "permissions" => rsx! { Placeholder { title: "Permissions", desc: "Role-based access control and policy editor.", icon: "lock" } },
            _ => rsx! { {admin_overview()} },
        },
    }
}

