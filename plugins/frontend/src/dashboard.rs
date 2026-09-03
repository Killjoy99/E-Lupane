//! E-Lupane dashboard — a Material Design 3 (light) e-learning dashboard with
//! four role personas: Student, Lecturer, Developer and Admin.

use dioxus::prelude::*;

use crate::assignments_ui::*;
use crate::components::*;
use crate::content_ui::CourseModules;
use crate::course_types::CourseView;
use crate::courses_ui::*;
use crate::enrollments_ui::*;
use crate::mock_data::NOTIFICATIONS;
use crate::mock_pages::*;
use crate::server_functions::{auth_login, auth_verify, courses_list, AuthSession};

const LOGO: Asset = asset!("/assets/logo.png");

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
    pub(crate) fn label(self) -> &'static str {
        match self {
            Role::Student => "Student",
            Role::Lecturer => "Lecturer",
            Role::Developer => "Developer",
            Role::Admin => "Admin",
        }
    }

    pub(crate) fn icon(self) -> &'static str {
        match self {
            Role::Student => "school",
            Role::Lecturer => "co_present",
            Role::Developer => "code_blocks",
            Role::Admin => "admin_panel_settings",
        }
    }
}

pub(crate) struct Persona {
    name: &'static str,
    subtitle: &'static str,
    initials: &'static str,
    tone: &'static str,
}

pub(crate) fn persona(role: Role) -> &'static Persona {
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

pub(crate) fn greeting(role: Role) -> &'static str {
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
            NavItem { label: "My Enrollments", icon: "manage_accounts", view: "enrollments" },
            NavItem { label: "Timetable", icon: "calendar_month", view: "timetable" },
            NavItem { label: "Content", icon: "video_library", view: "content" },
            NavItem { label: "Live Classes", icon: "video_camera_front", view: "live_classes" },
            NavItem { label: "Assignments", icon: "assignment", view: "assignments" },
            NavItem { label: "Grades", icon: "grade", view: "grades" },
            NavItem { label: "Certificates", icon: "workspace_premium", view: "certificates" },
            NavItem { label: "Messages", icon: "chat_bubble", view: "messages" },
        ],
                Role::Lecturer => &[
            NavItem { label: "Dashboard", icon: "space_dashboard", view: "overview" },
            NavItem { label: "My Courses", icon: "menu_book", view: "courses" },
            NavItem { label: "Enrollments", icon: "manage_accounts", view: "enrollments" },
            NavItem { label: "Assignments", icon: "assignment", view: "assignments" },
            NavItem { label: "Students", icon: "groups", view: "students" },
            NavItem { label: "Grade Book", icon: "fact_check", view: "gradebook" },
            NavItem { label: "Content", icon: "video_library", view: "content" },
            NavItem { label: "Live Classes", icon: "video_camera_front", view: "live_classes" },
            NavItem { label: "Analytics", icon: "monitoring", view: "analytics" },
        ],
        Role::Developer => &[
            NavItem { label: "Dashboard", icon: "space_dashboard", view: "overview" },
            NavItem { label: "Plugins", icon: "extension", view: "plugins" },
            NavItem { label: "SDKs", icon: "inventory_2", view: "sdks" },
            NavItem { label: "System Health", icon: "monitor_heart", view: "system" },
            NavItem { label: "Webhooks", icon: "webhook", view: "webhooks" },
            NavItem { label: "API Docs", icon: "description", view: "docs" },
        ],
                Role::Admin => &[
            NavItem { label: "Dashboard", icon: "space_dashboard", view: "overview" },
            NavItem { label: "Courses", icon: "menu_book", view: "courses" },
            NavItem { label: "Enrollments", icon: "manage_accounts", view: "enrollments" },
            NavItem { label: "Live Classes", icon: "video_camera_front", view: "live_classes" },
            NavItem { label: "Assignments", icon: "assignment", view: "assignments" },
            NavItem { label: "Users", icon: "group", view: "users" },
            NavItem { label: "Institutions", icon: "account_balance", view: "institutions" },
            NavItem { label: "Plugins", icon: "extension", view: "plugins" },
            NavItem { label: "Permissions", icon: "lock", view: "permissions" },
            NavItem { label: "Audit Log", icon: "receipt_long", view: "audit" },
        ],
    }
}

// --- App-wide UI context ----------------------------------------------------
// `pub(crate)` so the assignment/course UI modules can access the session.

#[derive(Clone, Copy)]
pub(crate) struct UiCtx {
    pub(crate) search: Signal<String>,
    pub(crate) notif_open: Signal<bool>,
    pub(crate) live_lesson: Signal<Option<&'static str>>,
    pub(crate) theme: Signal<&'static str>,
    /// The signed-in user (from the auth plugin), `None` while signed out.
    pub(crate) session: Signal<Option<AuthSession>>,
}

// ---------------------------------------------------------------------------
// App shell
// ---------------------------------------------------------------------------

/// Top-level dashboard: role-aware app shell + content routing.
pub fn Dashboard() -> Element {
    let role = use_signal(|| Role::Student);
    let active_view = use_signal(|| "overview");

    // NOTE: hooks must be created at the top level of the component, never inside
    // another hook's closure. Calling `use_signal` inside `use_context_provider`'s
    // closure would violate the rules of hooks and panic at runtime
    // ("The hook list is already borrowed").
    let search = use_signal(String::new);
    let notif_open = use_signal(|| false);
    let live_lesson = use_signal(|| None);
    let theme = use_signal(|| "light");

    // Auth state. `session` is `None` while signed out; it is restored from a
    // token persisted in `localStorage` once the app mounts in the browser.
    let session: Signal<Option<AuthSession>> = use_signal(|| None);
    let auth_checked = use_signal(|| false);

    let ctx = use_context_provider(move || UiCtx {
        search,
        notif_open,
        live_lesson,
        theme,
        session: session,
    });
    // Restore the user's saved theme once the app is mounted in the browser.
    // (On the server this eval is a no-op, so SSR is unaffected.)
    {
        let mut theme_sig = theme;
        use_future(move || async move {
            let eval = dioxus::document::eval(
                "try { return localStorage.getItem('elupane-theme') || 'light'; } catch (e) { return 'light'; }",
            );
            if let Ok(saved) = eval.join::<String>().await {
                theme_sig.set(if saved == "dark" { "dark" } else { "light" });
            }
        });
    }

    // Apply the active theme to `<html data-theme>` and persist it. This effect
    // re-runs on every render, so it always mirrors the current `theme` signal.
    use_effect(move || {
        let t = theme();
        let _ = dioxus::document::eval(&format!(
            "document.documentElement.setAttribute('data-theme','{t}');\
             try{{localStorage.setItem('elupane-theme','{t}');}}catch(e){{}}"
        ));
    });

    // Restore the signed-in session: read the stored token, validate it with
    // the auth plugin (Redis RPC on the server), and rebuild the session.
    // On the server this eval is a no-op, so SSR renders the loading state.
    {
        let mut session = session;
        let mut auth_checked = auth_checked;
        let mut role = role;
        use_future(move || async move {
            let eval = dioxus::document::eval(
                "try { return localStorage.getItem('elupane-token') || ''; } catch (e) { return ''; }",
            );
            let token = eval.join::<String>().await.unwrap_or_default();
            if !token.is_empty() {
                match auth_verify(token).await {
                    Ok(restored) => {
                        role.set(role_for_roles(&restored.user.roles));
                        session.set(Some(restored));
                    }
                    Err(_) => {
                        // Token expired/invalid — drop it so the next visit
                        // starts fresh at the login screen.
                        let _ = dioxus::document::eval(
                            "try { localStorage.removeItem('elupane-token'); } catch (e) {}",
                        );
                    }
                }
            }
            auth_checked.set(true);
        });
    }

    let query = (ctx.search)();
    let live = (ctx.live_lesson)();

    // Auth gate: wait for the session check, then show login or the app.
    if !auth_checked() {
        return rsx! {
            div { class: "auth-screen",
                div { class: "auth-card auth-loading-card",
                    span { class: "auth-spinner" }
                    p { "Restoring session…" }
                }
            }
        };
    }
    if session().is_none() {
        return rsx! {
            Login { session, role }
        };
    }

    rsx! {
        div { class: "md-app",
            Sidebar { role, active_view }
            div { class: "md-main",
                TopBar { role, active_view }
                main { class: "md-content", {render_page(role(), active_view(), &query, live)} }
            }
        }
    }
}

#[component]
fn Sidebar(role: Signal<Role>, active_view: Signal<&'static str>) -> Element {
    let ctx = use_context::<UiCtx>();
    let current = role();
    let (drawer_name, drawer_sub) = match (ctx.session)() {
        Some(s) => (
            s.user.full_name,
            if s.user.roles.is_empty() {
                current.label().to_string()
            } else {
                s.user.roles.join(", ")
            },
        ),
        None => (
            persona(current).name.to_string(),
            current.label().to_string(),
        ),
    };
    rsx! {
        aside { class: "md-drawer",
            div { class: "md-brand",
                div { class: "md-brand-logo",
                    img { src: LOGO, alt: "E-Lupane logo" }
                }
                div {
                    div { class: "md-brand-name", "E-Lupane" }
                    div { class: "md-brand-sub", "LL-OS" }
                }
            }

            div { class: "md-nav",
                p { class: "md-nav-title", "MENU" }
                for item in nav_items(current) {
                    button {
                        class: format!("md-nav-item {}", if active_view() == item.view { "active" } else { "" }),
                        onclick: move |_| active_view.set(item.view),
                        span { class: "material-symbols-outlined", "{item.icon}" }
                        span { "{item.label}" }
                    }
                }
            }
        
        // div { class: "md-drawer-user",
        //     UserAvatar { name: drawer_name.clone(), tone: persona(current).tone }
        //     div {
        //         div { class: "md-drawer-user-name", "{drawer_name}" }
        //         div { class: "md-drawer-user-sub", "{drawer_sub}" }
        //     }
        // }
        }
    }
}

#[component]
fn TopBar(role: Signal<Role>, active_view: Signal<&'static str>) -> Element {
    let current = role();
    let mut ctx = use_context::<UiCtx>();
    let mut notif_open = ctx.notif_open;
    let session = (ctx.session)();
    let (_user_name, _user_sub) = match &session {
        Some(s) => (
            s.user.full_name.clone(),
            if s.user.roles.is_empty() {
                current.label().to_string()
            } else {
                s.user.roles.join(", ")
            },
        ),
        None => (
            persona(current).name.to_string(),
            persona(current).subtitle.to_string(),
        ),
    };
    let mut session = ctx.session;

    // Prefer the signed-in user from the auth plugin; fall back to the persona
    // of the active role view (used before sign-in / for demo personas).
    let authed = session().map(|s| s.user);
    let display_name = authed
        .as_ref()
        .map(|u| u.full_name.clone())
        .unwrap_or_else(|| persona(current).name.to_string());
    let display_sub = authed
        .as_ref()
        .map(|u| {
            if u.roles.is_empty() {
                u.email.clone()
            } else {
                u.roles.join(" · ")
            }
        })
        .unwrap_or_else(|| persona(current).subtitle.to_string());
    let initials = initials_of(&display_name);

    rsx! {
        header { class: "md-topbar",
            div { class: "md-search",
                span { class: "material-symbols-outlined", "search" }
                input {
                    placeholder: "Search courses, people, plugins…",
                    value: (ctx.search)(),
                    oninput: move |e| ctx.search.set(e.value()),
                }
                if !(ctx.search)().is_empty() {
                    button {
                        class: "md-icon-btn",
                        style: "width: 30px; height: 30px; font-size: 18px",
                        onclick: move |_| ctx.search.set(String::new()),
                        span { class: "material-symbols-outlined", "close" }
                    }
                }
            }

            div { class: "md-topbar-spacer" }

            div { class: "md-topbar-role",
                span { class: "material-symbols-outlined", "{current.icon()}" }
                span { "{current.label()} view" }
            }

            button {
                class: "md-icon-btn",
                title: if (ctx.theme)() == "dark" { "Switch to light theme" } else { "Switch to dark theme" },
                onclick: move |_| {
                    let next = if (ctx.theme)() == "dark" { "light" } else { "dark" };
                    (ctx.theme).set(next);
                },
                span { class: "material-symbols-outlined",
                    if (ctx.theme)() == "dark" {
                        "light_mode"
                    } else {
                        "dark_mode"
                    }
                }
            }

            div { style: "position: relative",
                button {
                    class: "md-icon-btn",
                    onclick: move |_| notif_open.set(!notif_open()),
                    span { class: "material-symbols-outlined", "notifications" }
                    span { class: "md-badge", "5" }
                }
                if notif_open() {
                    div { class: "notif-panel",
                        div { class: "notif-head",
                            span { "Notifications" }
                            Button {
                                variant: "text",
                                onclick: move |_| notif_open.set(false),
                                "Clear all"
                            }
                        }
                        div { class: "list",
                            for & (icon , title , sub , tone) in NOTIFICATIONS {
                                div { class: "list-row",
                                    span { class: format!("icon-box icon-box-{tone}"),
                                        span { class: "material-symbols-outlined", "{icon}" }
                                    }
                                    div { class: "list-row-body",
                                        div { class: "list-row-title", "{title}" }
                                        div { class: "list-row-sub", "{sub}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "md-topbar-user",
                div { class: "md-avatar md-avatar-{persona(current).tone}", "{initials}" }
                div { class: "md-topbar-user-meta",
                    div { class: "md-topbar-user-name", "{display_name}" }
                    div { class: "md-topbar-user-sub", "{display_sub}" }
                }
                button {
                    class: "md-icon-btn",
                    title: "Sign out",
                    onclick: move |_| {
                        // Clear the stored token and drop the session — the
                        // auth gate will render the login screen again.
                        session.set(None);
                        let _ = dioxus::document::eval(
                            "try { localStorage.removeItem('elupane-token'); } catch (e) {}",
                        );
                    },
                    span { class: "material-symbols-outlined", "logout" }
                }
            }
        }
    }
}



// ---------------------------------------------------------------------------
// Content router
// ---------------------------------------------------------------------------


fn render_page(role: Role, view: &'static str, search: &str, live: Option<&'static str>) -> Element {
    if let Some(id) = live {
        return rsx! {
            LiveLesson { id, role }
        };
    }

    match role {
                Role::Student => match view {
            "courses" => rsx! {
                StudentCourses {}
            },
            "enrollments" => rsx! {
                StudentEnrollments {}
            },
            "timetable" => rsx! {
                {student_timetable()}
            },
            "content" => rsx! {
                StudentContent {}
            },
            "live_classes" => rsx! {
                crate::live_classes_ui::LiveClassesPage { can_manage: false }
            },
            "assignments" => rsx! {
                StudentAssignments {}
            },
            "grades" => rsx! {
                {student_grades()}
            },
            "certificates" => rsx! {
                {student_certificates()}
            },
            "messages" => rsx! {
                StudentMessages {}
            },
            _ => rsx! {
                {student_overview(search)}
            },
        },
                Role::Lecturer => match view {
            "courses" => rsx! {
                CourseManager {}
            },
            "enrollments" => rsx! {
                LecturerEnrollments {}
            },
            "assignments" => rsx! {
                LecturerAssignments {}
            },
            "students" => rsx! {
                {lecturer_students()}
            },
            "gradebook" => rsx! {
                {lecturer_gradebook()}
            },
            "content" => rsx! {
                LecturerContent {}
            },
            "live_classes" => rsx! {
                crate::live_classes_ui::LiveClassesPage { can_manage: true }
            },
            "analytics" => rsx! {
                {lecturer_analytics()}
            },
            _ => rsx! {
                {lecturer_overview()}
            },
        },
        Role::Developer => match view {
            "plugins" => rsx! {
                {developer_plugins(search)}
            },
            "sdks" => rsx! {
                {developer_sdks()}
            },
            "system" => rsx! {
                {developer_system()}
            },
            "webhooks" => rsx! {
                {developer_webhooks()}
            },
            "docs" => rsx! {
                {developer_docs(search)}
            },
            _ => rsx! {
                {DeveloperOverview()}
            },
        },
                Role::Admin => match view {
            "courses" => rsx! {
                CourseManager {}
            },
            "enrollments" => rsx! {
                LecturerEnrollments {}
            },
            "live_classes" => rsx! {
                crate::live_classes_ui::LiveClassesPage { can_manage: true }
            },
            "assignments" => rsx! {
                AdminAssignments {}
            },
            "users" => rsx! {
                {admin_users(search)}
            },
            "institutions" => rsx! {
                {admin_institutions(search)}
            },
            "plugins" => rsx! {
                AdminPlugins {}
            },
            "permissions" => rsx! {
                AdminPermissions {}
            },
            "audit" => rsx! {
                {admin_audit()}
            },
            _ => rsx! {
                {admin_overview()}
            },
        },
    }
}

// ---------------------------------------------------------------------------
// Authentication UI
// ---------------------------------------------------------------------------

/// Map the auth plugin's role names onto a dashboard role view.
fn role_for_roles(roles: &[String]) -> Role {
    if roles.iter().any(|r| r == "admin") {
        Role::Admin
    } else if roles.iter().any(|r| r == "lecturer") {
        Role::Lecturer
    } else if roles.iter().any(|r| r == "developer") {
        Role::Developer
    } else {
        Role::Student
    }
}

/// Avatar with initials derived from an arbitrary (runtime) name.
#[component]
fn UserAvatar(name: String, tone: &'static str) -> Element {
    let initials = initials_of(&name);
    rsx! {
        div { class: format!("md-avatar md-avatar-{tone}"), "{initials}" }
    }
}

/// Initials from a full name ("Thandiwe Ndlovu" → "TN").
fn initials_of(full_name: &str) -> String {
    let mut out = String::new();
    for word in full_name.split_whitespace().take(2) {
        if let Some(ch) = word.chars().next() {
            out.extend(ch.to_uppercase());
        }
    }
    if out.is_empty() {
        out.push('U');
    }
    out
}

/// Shared login routine: call the auth plugin via the server function, persist
/// the token, and enter the app with the user's primary role.
async fn perform_login(
    username: String,
    password: String,
    mut session: Signal<Option<AuthSession>>,
    mut role: Signal<Role>,
    mut error: Signal<Option<String>>,
    mut busy: Signal<bool>,
) {
    busy.set(true);
    error.set(None);
    match auth_login(username, password).await {
        Ok(s) => {
            // Persist the token so the session survives page reloads.
            let _ = dioxus::document::eval(&format!(
                "try {{ localStorage.setItem('elupane-token','{}'); }} catch (e) {{}}",
                s.token
            ));
            role.set(role_for_roles(&s.user.roles));
            session.set(Some(s));
        }
        Err(err) => error.set(Some(err.to_string())),
    }
    busy.set(false);
}

/// Full-screen sign-in: authenticates against the auth plugin over Redis.
#[component]
fn Login(session: Signal<Option<AuthSession>>, role: Signal<Role>) -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut busy: Signal<bool> = use_signal(|| false);

    rsx! {
        div { class: "auth-screen",
            div { class: "auth-card",
                div { class: "auth-logo",
                    img { src: LOGO, alt: "E-Lupane logo" }
                }
                h1 { class: "auth-title", "E-Lupane" }
                p { class: "auth-sub", "Lupane State University · Learning OS" }

                form {
                    class: "auth-form",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        async move {
                            if busy() {
                                return;
                            }
                            let user = username().trim().to_string();
                            if user.is_empty() || password().is_empty() {
                                error.set(Some("Enter your username and password.".into()));
                                return;
                            }
                            perform_login(user, password(), session, role, error, busy).await;
                        }
                    },

                    div { class: "auth-field",
                        label { "Username" }
                        input {
                            r#type: "text",
                            placeholder: "e.g. admin",
                            autocomplete: "username",
                            value: "{username}",
                            oninput: move |e| username.set(e.value()),
                        }
                    }
                    div { class: "auth-field",
                        label { "Password" }
                        input {
                            r#type: "password",
                            placeholder: "••••••••",
                            autocomplete: "current-password",
                            value: "{password}",
                            oninput: move |e| password.set(e.value()),
                        }
                    }

                    if let Some(err) = error() {
                        div { class: "auth-error",
                            span { class: "material-symbols-outlined", "error" }
                            span { "{err}" }
                        }
                    }

                    button {
                        class: "auth-submit",
                        r#type: "submit",
                        disabled: busy(),
                        if busy() {
                            "Signing in…"
                        } else {
                            "Sign in"
                        }
                    }
                }

                p { class: "auth-hint", "Authenticate via the auth service · Redis messaging" }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Content views (backed by the content plugin)
// ---------------------------------------------------------------------------

#[component]
pub fn StudentContent() -> Element {
    let ctx = use_context::<UiCtx>();
    let session = (ctx.session)();
    if let Some(s) = session {
        let mut selected: Signal<Option<CourseView>> = use_signal(|| None);
        let token = s.token.clone();

        rsx! {
            div { class: "page",
                SectionTitle {
                    title: "My learning content",
                    subtitle: "Select a course to view its modules and lessons",
                }
                if selected().is_none() {
                    MyCourseList {
                        token,
                        on_select: {
                            let mut selected = selected;
                            move |c| selected.set(Some(c))
                        },
                    }
                } else if let Some(c) = selected() {
                    div { class: "md-actions",
                        Button {
                            variant: "text",
                            icon: "arrow_back",
                            onclick: move |_| selected.set(None),
                            "Back to courses"
                        }
                    }
                    CourseModules { course_id: c.id, can_manage: false }
                }
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
pub fn LecturerContent() -> Element {
    let ctx = use_context::<UiCtx>();
    let session = (ctx.session)();
    if let Some(s) = session {
        let mut selected: Signal<Option<CourseView>> = use_signal(|| None);
        let token = s.token.clone();

        rsx! {
            div { class: "page",
                SectionTitle {
                    title: "Course content management",
                    subtitle: "Create modules, lessons and publish materials",
                }
                if selected().is_none() {
                    MyCourseList {
                        token,
                        on_select: {
                            let mut selected = selected;
                            move |c| selected.set(Some(c))
                        },
                    }
                } else if let Some(c) = selected() {
                    div { class: "md-actions",
                        Button {
                            variant: "text",
                            icon: "arrow_back",
                            onclick: move |_| selected.set(None),
                            "Back to courses"
                        }
                    }
                    CourseModules { course_id: c.id, can_manage: true }
                }
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
fn MyCourseList(
    token: String,
    on_select: EventHandler<CourseView>,
) -> Element {
    let mut refresh = use_signal(|| 0u32);
    let mut courses = use_resource(move || {
        let token = token.clone();
        async move {
            let _ = refresh();
            courses_list(token, Some("published".to_string()), false, None)
                .await
                .unwrap_or_default()
        }
    });
    let list_ref = courses.read();
    let list_opt: &Option<Vec<CourseView>> = &*list_ref;

    rsx! {
        if let Some(list) = list_opt {
            if list.is_empty() {
                div { class: "md-card placeholder",
                    span { class: "material-symbols-outlined placeholder-icon", "school" }
                    h3 { "No courses yet" }
                    p { "Courses appear here once published." }
                }
            } else {
                div { class: "grid-3",
                    for c in list.iter().cloned() {
                        div { class: "md-card course-card",
                            div { class: "course-card-head",
                                span { class: "course-icon course-icon-teal",
                                    span { class: "material-symbols-outlined", "menu_book" }
                                }
                                span { class: "course-code", "{c.code}" }
                            }
                            h3 { "{c.title}" }
                            p { class: "muted", style: "font-size: 13px", "{c.short_description}" }
                            Button {
                                variant: "tonal",
                                icon: "visibility",
                                onclick: move |_| on_select.call(c.clone()),
                                "View content"
                            }
                        }
                    }
                }
            }
        } else {
            div { class: "md-card placeholder",
                span { class: "material-symbols-outlined placeholder-icon", "hourglass_empty" }
                h3 { "Loading courses…" }
            }
        }
    }
}


