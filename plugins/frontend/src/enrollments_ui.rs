//! Enrollment UI components (live — backed by the enrollments plugin).
//!
//! Components:
//! * StudentEnrollments — the student's own enrollments.
//! * EnrollmentRoster   — an instructor/admin's course roster.
//! * EnrollmentEditor   — add student / bulk enroll / invite link.
//! * EnrollmentAccessGate — gate used inside a course page.

use dioxus::prelude::*;

use crate::components::*;
use crate::dashboard::UiCtx;
use crate::enrollment_types;
use crate::server_functions::{
    courses_list, enrollments_access, enrollments_action, enrollments_bulk, enrollments_enroll,
    enrollments_invite_create, enrollments_mine, enrollments_roster, enrollments_self,
};

use crate::enrollment_types::{AccessView, EnrollmentView};
use crate::course_types::CourseView;

/// Student view: the caller's own enrollments.
#[component]
pub fn StudentEnrollments() -> Element {
    let ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());
    let mut refresh = use_signal(|| 0u32);

    let token_fetch = token.clone();
    let mut enrl = use_resource(move || {
        let token = token_fetch.clone();
        async move {
            let _ = refresh();
            match token {
                Some(t) => enrollments_mine(t).await.unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });
    let list = enrl.read();
    let items: &Option<Vec<EnrollmentView>> = &*list;

    rsx! {
        div { class: "page",
            SectionTitle {
                title: "My enrollments",
                subtitle: "Courses you are taking, with your current status",
                action: rsx! { Button { variant: "tonal", icon: "refresh", onclick: move |_| refresh += 1, "Refresh" } }
            }
            if let Some(items) = items {
                if items.is_empty() {
                    div { class: "md-card placeholder",
                        span { class: "material-symbols-outlined placeholder-icon", "menu_book" }
                        h3 { "No enrollments yet" }
                        p { "Browse the published course catalogue and enroll in a course." }
                    }
                } else {
                    div { class: "md-card table-card",
                        table { class: "md-table",
                            thead { tr { th { "Course" } th { "State" } th { "Source" } th { "Expires" } } }
                            tbody {
                                for e in items.iter() {
                                    tr {
                                        td { "{e.course_id}" }
                                        td { StatusChip { status: enrollment_types::state_tone(e.state.as_str()).to_string(), label: Some(enrollment_types::state_label(e.state.as_str()).to_string()) } }
                                        td { "{e.source}" }
                                        td { if let Some(exp) = &e.expires_at { "{exp}" } else { "—" } }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "md-card placeholder",
                    span { class: "material-symbols-outlined placeholder-icon", "hourglass_empty" }
                    h3 { "Loading enrollments…" }
                }
            }
        }
    }
}

/// Instructor/admin roster for one course.
#[component]
pub fn EnrollmentRoster(course_id: uuid::Uuid) -> Element {
    let ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());
    let mut refresh = use_signal(|| 0u32);
    let mut filter = use_signal(|| String::new());
    let mut editor: Signal<Option<EditorMode>> = use_signal(|| None);

    let token_fetch = token.clone();
    let mut roster = use_resource(move || {
        let token = token_fetch.clone();
        let filt = filter();
        async move {
            let _ = refresh();
            match token {
                Some(t) => {
                    let filt_opt = if filt.is_empty() { None } else { Some(filt.clone()) };
                    enrollments_roster(t, course_id, filt_opt).await.unwrap_or_default()
                }
                None => Vec::new(),
            }
        }
    });
    let list = roster.read();
    let items: &Option<Vec<EnrollmentView>> = &*list;
    let on_close = move |_| editor.set(None);
    let on_saved = { let mut refresh = refresh; move |_| refresh += 1 };

    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Enrollments",
                subtitle: "Students enrolled in this course",
                action: rsx! { Button { variant: "tonal", icon: "refresh", onclick: move |_| refresh += 1, "Refresh" } }
            }
            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    input { class: "md-input md-input-sm", placeholder: "Filter by state (pending/active/...)", value: "{filter}", oninput: move |e| filter.set(e.value()) }
                    Button { variant: "filled", icon: "person_add", onclick: move |_| editor.set(Some(EditorMode::AddStudent)), "Add student" }
                    Button { variant: "outlined", icon: "selection_multiple_checkbox", onclick: move |_| editor.set(Some(EditorMode::Bulk)), "Bulk" }
                    Button { variant: "outlined", icon: "link", onclick: move |_| editor.set(Some(EditorMode::Invite)), "Invite link" }
                }
                if let Some(items) = items {
                    if items.is_empty() {
                        div { class: "md-card placeholder",
                            span { class: "material-symbols-outlined placeholder-icon", "groups_2" }
                            h3 { "No enrollments" }
                            p { "Use 'Add student' to enroll learners into this course." }
                        }
                    } else {
                        table { class: "md-table",
                            thead { tr { th { "Student" } th { "State" } th { "Source" } th { "Enrolled" } th { "Actions" } } }
                            tbody {
                                for e in items.iter() {
                                    tr {
                                        td { "{e.student.full_name} ({e.student.username})" }
                                        td { StatusChip { status: enrollment_types::state_tone(e.state.as_str()).to_string(), label: Some(enrollment_types::state_label(e.state.as_str()).to_string()) } }
                                        td { "{e.source}" }
                                        td { "{e.created_at}" }
                                        td { EnrollmentManagerActions { enrollment: e.clone(), on_change: { let mut refresh = refresh; move |_| refresh += 1 } } }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "md-card placeholder",
                        span { class: "material-symbols-outlined placeholder-icon", "hourglass_empty" }
                        h3 { "Loading roster…" }
                    }
                }
            }
            if let Some(mode) = editor() {
                EnrollmentEditor { course_id, mode, on_close, on_saved }
            }
        }
    }
}

/// Manager action buttons for an enrollment row.
#[component]
fn EnrollmentManagerActions(enrollment: EnrollmentView, on_change: EventHandler) -> Element {
    let token = (use_context::<UiCtx>().session)()
        .map(|s| s.token.clone());
    let eid = enrollment.id;
    let current = enrollment.state.clone();

    rsx! {
        div { class: "row-actions",
            for (act, label) in enrollment_types::available_transitions(current.as_str()) {
                Button {
                    variant: "text",
                    onclick: {
                        let token = token.clone();
                        let on_change = on_change.clone();
                        move |_| {
                            if let Some(tok) = &token {
                                let tok = tok.clone();
                                spawn(async move {
                                    let _ = enrollments_action(tok, act.to_string(), eid, None).await;
                                    on_change.call(());
                                });
                            }
                        }
                    },
                    "{label}"
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditorMode {
    AddStudent,
    Bulk,
    Invite,
}



/// Modal editor for add student / bulk enroll / invite link.
#[component]
fn EnrollmentEditor(
    course_id: uuid::Uuid,
    mode: EditorMode,
    on_close: EventHandler,
    on_saved: EventHandler,
) -> Element {
    let ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());
    let mut student = use_signal(String::new);
    let mut students_blob = use_signal(String::new);
    let mut max_uses = use_signal(String::new);
    let mut busy: Signal<bool> = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let title = match mode {
        EditorMode::AddStudent => "Enroll a student",
        EditorMode::Bulk => "Bulk enroll students",
        EditorMode::Invite => "Create invite link",
    };

    rsx! {
        div { class: "md-overlay",
            div { class: "md-card editor-card",
                h3 { "{title}" }
                if mode == EditorMode::AddStudent {
                    div { class: "enroll-field field-full",
                        label { "Student username or email" }
                        input { placeholder: "e.g. thandiwe.n or thandiwe@elupane.edu", value: "{student}", oninput: move |e| student.set(e.value()) }
                    }
                }
                if mode == EditorMode::Bulk {
                    div { class: "enroll-field field-full",
                        label { "Students (one username or email per line)" }
                        textarea { rows: "5", value: "{students_blob}", oninput: move |e| students_blob.set(e.value()) }
                    }
                }
                if mode == EditorMode::Invite {
                    div { class: "enroll-field",
                        label { "Max uses (leave blank for unlimited)" }
                        input { placeholder: "e.g. 10", value: "{max_uses}", oninput: move |e| max_uses.set(e.value()) }
                    }
                }
                if let Some(err) = error() {
                    div { class: "auth-error", span { class: "material-symbols-outlined", "error" } span { "{err}" } }
                }
                div { class: "submission-actions",
                    Button {
                        icon: "save",
                        disabled: busy(),
                        onclick: move |_| {
                            let Some(token) = token.clone() else { return; };
                            let on_saved = on_saved;
                            let on_close = on_close;
                            let mut busy = busy;
                            let mut error = error;
                            let student = student();
                            let students_blob = students_blob();
                            let max_uses = max_uses();
                            spawn(async move {
                                busy.set(true);
                                error.set(None);
                                let res = if mode == EditorMode::Bulk {
                                    let list: Vec<String> = students_blob
                                        .lines().map(|l| l.trim().to_string())
                                        .filter(|l| !l.is_empty()).collect();
                                    enrollments_bulk(token, course_id, list, None).await
                                        .map(|_| ())
                                } else if mode == EditorMode::Invite {
                                    let mu = max_uses.parse::<i64>().ok();
                                    enrollments_invite_create(token, course_id, None, mu).await
                                        .map(|_| ())
                                } else {
                                    enrollments_enroll(token, course_id, student, None).await
                                        .map(|_| ())
                                };
                                match res {
                                    Ok(_) => { on_saved.call(()); on_close.call(()); }
                                    Err(e) => error.set(Some(e.to_string())),
                                }
                                busy.set(false);
                            });
                        },
                        "Save"
                    }
                    Button { variant: "text", icon: "close", onclick: move |_| on_close.call(()), "Cancel" }
                }
            }
        }
    }
}

/// Gate used inside a course page: renders `children` only when the caller may
/// access the course (active enrollment, or a manager). Otherwise shows an
/// enrol-self prompt.
#[component]
pub fn EnrollmentAccessGate(course_id: uuid::Uuid, children: Element) -> Element {
    let ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());
    let mut access: Signal<Option<AccessView>> = use_signal(|| None);

    {
        let token_check = token.clone();
        use_resource(move || {
            let token = token_check.clone();
            async move {
                if let Some(t) = token {
                    access.set(Some(
                        enrollments_access(t, course_id).await.unwrap_or(AccessView {
                            course_id,
                            allowed: false,
                            state: None,
                            manager: false,
                        }),
                    ));
                }
            }
        });
    }

    if let Some(a) = access() {
        if a.allowed {
            return rsx! { {children} };
        }
        return rsx! {
            div { class: "md-card placeholder",
                span { class: "material-symbols-outlined placeholder-icon", "lock" }
                h3 { "Enroll to access this course" }
                p { "You must have an active enrollment to view this content." }
                if let Some(t) = token.clone() {
                    Button {
                        icon: "menu_book",
                        onclick: move |_| {
                            let t = t.clone();
                            spawn(async move { let _ = enrollments_self(t, course_id).await; });
                        },
                        "Enroll myself"
                    }
                }
            }
        };
    }

    rsx! {
        div { class: "md-card placeholder",
            span { class: "material-symbols-outlined placeholder-icon", "hourglass_empty" }
            h3 { "Checking access…" }
        }
    }
}

/// Instructor/admin entry point: lists the caller's courses, then shows the
/// roster for the selected one.
#[component]
pub fn LecturerEnrollments() -> Element {
    let ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());
    let mut refresh = use_signal(|| 0u32);
    let mut selected: Signal<Option<CourseView>> = use_signal(|| None);

    let token_fetch = token.clone();
    let mut courses = use_resource(move || {
        let token = token_fetch.clone();
        async move {
            let _ = refresh();
            match token {
                Some(t) => {
                    courses_list(t, None, true, None).await.unwrap_or_default()
                }
                None => Vec::new(),
            }
        }
    });
    let list = courses.read();
    let items: &Option<Vec<CourseView>> = &*list;

    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Course enrollments",
                subtitle: "Select a course to manage its enrollments",
                action: rsx! { Button { variant: "tonal", icon: "refresh", onclick: move |_| refresh += 1, "Refresh" } }
            }
            if let Some(items) = items {
                if items.is_empty() {
                    div { class: "md-card placeholder",
                        span { class: "material-symbols-outlined placeholder-icon", "menu_book" }
                        h3 { "No courses to manage" }
                        p { "Create or own a course to see its enrollments here." }
                    }
                } else {
                    div { class: "grid-3",
                        for c in items.iter().cloned() {
                            div {
                                onclick: move |_| selected.set(Some(c.clone())),
                                class: "md-card course-card",
                                div { class: "course-card-head",
                                    span { class: "course-icon course-icon-indigo",
                                        span { class: "material-symbols-outlined", "menu_book" }
                                    }
                                    div {
                                        div { class: "course-card-title", "{c.title}" }
                                        div { class: "course-card-code", "{c.code}" }
                                    }
                                    span { class: "material-symbols-outlined status-action", "chevron_right" }
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
            { if let Some(c) = selected() {
                rsx! {
                    div { class: "md-overlay",
                        div { class: "md-card editor-card",
                            h3 { "Manage: {c.title} ({c.code})" }
                            EnrollmentRoster { course_id: c.id }
                            Button { variant: "text", icon: "close", onclick: move |_| selected.set(None), "Back" }
                        }
                    }
                }
            } else {
                rsx! {}
            } }
        }
    }
}

