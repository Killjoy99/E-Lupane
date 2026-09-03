//! Learning content UI (live — backed by the content plugin).

use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::*;
use crate::content_types::*;
use crate::dashboard::UiCtx;
use crate::server_functions::*;

// ---------------------------------------------------------------------------
// Shared: module list for a course
// ---------------------------------------------------------------------------

#[component]
pub fn CourseModules(course_id: Uuid, can_manage: bool) -> Element {
    let mut ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());
    let token_fetch = token.clone();
    let mut refresh = use_signal(|| 0u32);
    let mut creating = use_signal(|| false);
    let mut expanded: Signal<Option<Uuid>> = use_signal(|| None);

    let mut modules = use_resource(move || {
        let token = token_fetch.clone();
        async move {
            let _ = refresh();
            match token {
                Some(t) => content_modules_list(t, course_id).await.unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });

    let list_ref = modules.read();
    let list_opt: &Option<Vec<ContentModuleView>> = &*list_ref;
    let action = if can_manage {
        Some(rsx! {
            Button {
                variant: "filled",
                icon: "add",
                onclick: move |_| creating.set(true),
                "New module"
            }
        })
    } else {
        None
    };

    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Course content",
                subtitle: "Modules, lessons and learning materials",
                action: action,
            }
            if creating() {
                CreateModuleForm {
                    course_id,
                    on_created: move |_| { refresh += 1; creating.set(false); },
                    on_cancel: move |_| creating.set(false),
                }
            }
            if let Some(list) = list_opt {
                if list.is_empty() {
                    div { class: "md-card placeholder",
                        span { class: "material-symbols-outlined placeholder-icon", "menu_book" }
                        h3 { "No modules yet" }
                        p { "Add modules to structure your course content." }
                    }
                } else {
                    div { class: "content-tree",
                        for m in list.iter().cloned() {
                            ModuleCard {
                                module: m,
                                course_id,
                                can_manage,
                                expanded: expanded,
                                on_refresh: move |_| refresh += 1,
                            }
                        }
                    }
                }
            } else {
                div { class: "md-card placeholder",
                    span { class: "material-symbols-outlined placeholder-icon", "hourglass_empty" }
                    h3 { "Loading content…" }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Module card
// ---------------------------------------------------------------------------

#[component]
pub fn ModuleCard(
    module: ContentModuleView,
    course_id: Uuid,
    can_manage: bool,
    expanded: Signal<Option<Uuid>>,
    on_refresh: EventHandler<()>,
) -> Element {
    let is_open = expanded() == Some(module.id);
    let mut local = use_signal(|| module.clone());
    let m = local();
    let m_for_closure = m.clone();

    rsx! {
        div { class: "module-card",
            div { class: "module-card-head",
                button {
                    class: "md-icon-btn",
                    style: "width: 36px; height: 36px",
                    onclick: move |_| {
                        if expanded() == Some(module.id) {
                            expanded.set(None);
                        } else {
                            expanded.set(Some(module.id));
                        }
                    },
                    span { class: "material-symbols-outlined", if is_open { "expand_more" } else { "chevron_right" } }
                }
                div { class: "module-info",
                    h3 { "{m.title}" }
                    span { class: "muted", style: "font-size: 13px", "{m.description}" }
                }
                StatusChip { status: m.status.clone(), label: Some(m.status.clone()) }
                if can_manage {
                    ModuleActions {
                        module: m.clone(),
                        on_changed: move |_| { local.set(m_for_closure.clone()); on_refresh.call(()); },
                    }
                }
            }
            if is_open {
                ModuleLessons {
                    module_id: m.id,
                    course_id,
                    can_manage,
                    on_refresh: move |_| on_refresh.call(()),
                }
            }
        }
    }
}

#[component]
fn ModuleActions(
    module: ContentModuleView,
    on_changed: EventHandler<()>,
) -> Element {
    let mut show_menu = use_signal(|| false);
    let mut show_edit = use_signal(|| false);
    let mut show_status = use_signal(|| false);
    let token = (use_context::<UiCtx>().session)().map(|s| s.token.clone());
    let token_for_status = token.clone();

    rsx! {
        div { class: "md-actions",
            button {
                class: "md-icon-btn",
                onclick: move |_| show_menu.set(!show_menu()),
                span { class: "material-symbols-outlined", "more_vert" }
            }
            if show_menu() {
                div { class: "md-menu",
                    button {
                        onclick: move |_| { show_menu.set(false); show_edit.set(true); },
                        span { class: "material-symbols-outlined", "edit" }
                        span { "Edit" }
                    }
                    if module.status == "draft" || module.status == "unpublished" {
                        button {
                            onclick: move |_| {
                                show_menu.set(false);
                                show_status.set(true);
                            },
                            span { class: "material-symbols-outlined", "publish" }
                            span { "Publish" }
                        }
                    } else {
                        button {
                            onclick: move |_| {
                                show_menu.set(false);
                                show_status.set(true);
                            },
                            span { class: "material-symbols-outlined", "unpublished" }
                            span { "Unpublish" }
                        }
                    }
                    button {
                        class: "danger",
                        onclick: move |_| {
                            show_menu.set(false);
                            if let Some(t) = &token {
                                let mid = module.id;
                                let t = t.clone();
                                spawn(async move {
                                    let _ = content_modules_delete(t, mid).await;
                                    on_changed.call(());
                                });
                            }
                        },
                        span { class: "material-symbols-outlined", "delete" }
                        span { "Delete" }
                    }
                }
            }
            if show_edit() {
                EditModuleDialog {
                    module: module.clone(),
                    on_saved: move |_| { show_edit.set(false); on_changed.call(()); },
                    on_cancel: move |_| show_edit.set(false),
                }
            }
            if show_status() {
                SetStatusDialog {
                    current: module.status.clone(),
                    on_set: move |status: String| {
                        show_status.set(false);
                        if let Some(t) = &token_for_status {
                            let mid = module.id;
                            let t = t.clone();
                            let status = status.clone();
                            spawn(async move {
                                let _ = content_modules_status(t, mid, status).await;
                                on_changed.call(());
                            });
                        }
                    },
                    on_cancel: move |_| show_status.set(false),
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Lesson list inside a module
// ---------------------------------------------------------------------------

#[component]
pub fn ModuleLessons(
    module_id: Uuid,
    course_id: Uuid,
    can_manage: bool,
    on_refresh: EventHandler<()>,
) -> Element {
    let token = (use_context::<UiCtx>().session)().map(|s| s.token.clone());
    let token_fetch = token.clone();
    let mut refresh = use_signal(|| 0u32);
    let mut creating = use_signal(|| false);

    let mut lessons = use_resource(move || {
        let token = token_fetch.clone();
        async move {
            let _ = refresh();
            match token {
                Some(t) => content_lessons_list(t, module_id).await.unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });

    let list_ref = lessons.read();
    let list_opt: &Option<Vec<ContentLessonView>> = &*list_ref;

    rsx! {
        div { class: "lessons-list",
            if can_manage {
                div { class: "md-actions",
                    if creating() {
                        button {
                            class: "md-btn md-btn-text",
                            onclick: move |_| creating.set(false),
                            "Cancel"
                        }
                    } else {
                        Button {
                            variant: "tonal",
                            icon: "add",
                            onclick: move |_| creating.set(true),
                            "New lesson"
                        }
                    }
                }
            }
            if creating() {
                CreateLessonForm {
                    module_id,
                    on_created: move |_| { refresh += 1; creating.set(false); on_refresh.call(()); },
                    on_cancel: move |_| creating.set(false),
                }
            }
            if let Some(list) = list_opt {
                if list.is_empty() {
                    p { class: "muted", "No lessons yet." }
                } else {
                    for l in list.iter().cloned() {
                        LessonRow {
                            lesson: l,
                            can_manage,
                            on_changed: move |_| { refresh += 1; on_refresh.call(()); },
                        }
                    }
                }
            } else {
                p { class: "muted", "Loading lessons…" }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Lesson row
// ---------------------------------------------------------------------------

#[component]
pub fn LessonRow(
    lesson: ContentLessonView,
    can_manage: bool,
    on_changed: EventHandler<()>,
) -> Element {
    let mut show_edit = use_signal(|| false);
    let mut show_status = use_signal(|| false);
    let token = (use_context::<UiCtx>().session)().map(|s| s.token.clone());

    rsx! {
        div { class: "lesson-row",
            span { class: "material-symbols-outlined lesson-icon", "play_circle" }
            div { class: "lesson-info",
                h4 { "{lesson.title}" }
                span { class: "muted", style: "font-size: 13px", "{lesson.description}" }
            }
            StatusChip { status: lesson.status.clone(), label: Some(lesson.status.clone()) }
            if can_manage {
                div { class: "md-actions",
                    button {
                        class: "md-icon-btn",
                        onclick: move |_| show_edit.set(!show_edit()),
                        span { class: "material-symbols-outlined", "edit" }
                    }
                    button {
                        class: "md-icon-btn",
                        onclick: move |_| show_status.set(!show_status()),
                        span { class: "material-symbols-outlined", "publish" }
                    }
                }
            }
            if show_edit() {
                EditLessonDialog {
                    lesson: lesson.clone(),
                    on_saved: move |_| { show_edit.set(false); on_changed.call(()); },
                    on_cancel: move |_| show_edit.set(false),
                }
            }
            if show_status() {
                SetStatusDialog {
                    current: lesson.status.clone(),
                    on_set: move |status: String| {
                        show_status.set(false);
                        if let Some(t) = &token {
                            let lid = lesson.id;
                            let t = t.clone();
                            let status = status.clone();
                            spawn(async move {
                                let _ = content_lessons_status(t, lid, status).await;
                                on_changed.call(());
                            });
                        }
                    },
                    on_cancel: move |_| show_status.set(false),
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Forms
// ---------------------------------------------------------------------------

#[component]
pub fn CreateModuleForm(
    course_id: Uuid,
    on_created: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let token = (use_context::<UiCtx>().session)().map(|s| s.token.clone());

    rsx! {
        div { class: "md-card",
            h4 { "New module" }
            input {
                placeholder: "Module title",
                value: title(),
                oninput: move |e| title.set(e.value()),
            }
            textarea {
                placeholder: "Description",
                value: description(),
                oninput: move |e| description.set(e.value()),
            }
            if let Some(err) = error() {
                p { class: "error-text", "{err}" }
            }
            div { class: "md-actions",
                Button {
                    variant: "text",
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
                    Button {
                        variant: "filled",
                        disabled: busy() || title().trim().is_empty(),
                        onclick: move |_| {
                            busy.set(true);
                            error.set(None);
                            if let Some(t) = &token {
                                let course_id = course_id;
                                let title = title().trim().to_string();
                                let description = description().trim().to_string();
                                let t = t.clone();
                                spawn(async move {
                                    let _ = content_modules_create(t, CreateModuleInput {
                                        course_id,
                                        title,
                                        description,
                                        sort_order: 0,
                                    }).await;
                                    busy.set(false);
                                    on_created.call(());
                                });
                            } else {
                                busy.set(false);
                                error.set(Some("Not authenticated".to_string()));
                            }
                        },
                        if busy() { "Saving…" } else { "Create module" }
                    }
            }
        }
    }
}

#[component]
fn EditModuleDialog(
    module: ContentModuleView,
    on_saved: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut title = use_signal(|| module.title.clone());
    let mut description = use_signal(|| module.description.clone());
    let mut busy = use_signal(|| false);
    let token = (use_context::<UiCtx>().session)().map(|s| s.token.clone());

    rsx! {
        div { class: "md-dialog-backdrop",
            div { class: "md-dialog",
                h3 { "Edit module" }
                input {
                    placeholder: "Title",
                    value: title(),
                    oninput: move |e| title.set(e.value()),
                }
                textarea {
                    placeholder: "Description",
                    value: description(),
                    oninput: move |e| description.set(e.value()),
                }
                div { class: "md-actions",
                    Button {
                        variant: "text",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    Button {
                        variant: "filled",
                        disabled: busy() || title().trim().is_empty(),
                        onclick: move |_| {
                            busy.set(true);
                            if let Some(t) = &token {
                                let module = module.clone();
                                let title = title().trim().to_string();
                                let description = description().trim().to_string();
                                let t = t.clone();
                                spawn(async move {
                                    let _ = content_modules_update(t, UpdateModuleInput {
                                        module_id: module.id,
                                        title: Some(title),
                                        description: Some(description),
                                        sort_order: None,
                                    }).await;
                                    busy.set(false);
                                    on_saved.call(());
                                });
                            } else {
                                busy.set(false);
                            }
                        },
                        if busy() { "Saving…" } else { "Save" }
                    }
                }
            }
        }
    }
}

#[component]
fn CreateLessonForm(
    module_id: Uuid,
    on_created: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let token = (use_context::<UiCtx>().session)().map(|s| s.token.clone());

    rsx! {
        div { class: "md-card",
            h4 { "New lesson" }
            input {
                placeholder: "Lesson title",
                value: title(),
                oninput: move |e| title.set(e.value()),
            }
            textarea {
                placeholder: "Description",
                value: description(),
                oninput: move |e| description.set(e.value()),
            }
            if let Some(err) = error() {
                p { class: "error-text", "{err}" }
            }
            div { class: "md-actions",
                Button {
                    variant: "text",
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
                Button {
                    variant: "filled",
                    disabled: busy() || title().trim().is_empty(),
                    onclick: move |_| {
                        busy.set(true);
                        error.set(None);
                        if let Some(t) = &token {
                            let module_id = module_id;
                            let title = title().trim().to_string();
                            let description = description().trim().to_string();
                            let t = t.clone();
                            spawn(async move {
                                let _ = content_lessons_create(t, CreateLessonInput {
                                    module_id,
                                    title,
                                    description,
                                    sort_order: 0,
                                }).await;
                                busy.set(false);
                                on_created.call(());
                            });
                        } else {
                            busy.set(false);
                            error.set(Some("Not authenticated".to_string()));
                        }
                    },
                    if busy() { "Saving…" } else { "Create lesson" }
                }
            }
        }
    }
}

#[component]
fn EditLessonDialog(
    lesson: ContentLessonView,
    on_saved: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut title = use_signal(|| lesson.title.clone());
    let mut description = use_signal(|| lesson.description.clone());
    let mut busy = use_signal(|| false);
    let token = (use_context::<UiCtx>().session)().map(|s| s.token.clone());

    rsx! {
        div { class: "md-dialog-backdrop",
            div { class: "md-dialog",
                h3 { "Edit lesson" }
                input {
                    placeholder: "Title",
                    value: title(),
                    oninput: move |e| title.set(e.value()),
                }
                textarea {
                    placeholder: "Description",
                    value: description(),
                    oninput: move |e| description.set(e.value()),
                }
                div { class: "md-actions",
                    Button {
                        variant: "text",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    Button {
                        variant: "filled",
                        disabled: busy() || title().trim().is_empty(),
                        onclick: move |_| {
                            busy.set(true);
                        if let Some(t) = &token {
                            let lesson = lesson.clone();
                            let title = title().trim().to_string();
                            let description = description().trim().to_string();
                            let t = t.clone();
                            spawn(async move {
                                let _ = content_lessons_update(t, UpdateLessonInput {
                                    lesson_id: lesson.id,
                                    title: Some(title),
                                    description: Some(description),
                                    sort_order: None,
                                }).await;
                                busy.set(false);
                                on_saved.call(());
                            });
                        } else {
                            busy.set(false);
                        }
                        },
                        if busy() { "Saving…" } else { "Save" }
                    }
                }
            }
        }
    }
}

#[component]
fn SetStatusDialog(
    current: String,
    on_set: EventHandler<String>,
    on_cancel: EventHandler<()>,
) -> Element {
    let target = if current == "draft" || current == "unpublished" {
        "published"
    } else {
        "unpublished"
    };

    rsx! {
        div { class: "md-dialog-backdrop",
            div { class: "md-dialog",
                h3 { if target == "published" { "Publish?" } else { "Unpublish?" } }
                p { "Change status to \"{target}\"?" }
                div { class: "md-actions",
                    Button {
                        variant: "text",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    Button {
                        variant: "filled",
                        onclick: move |_| on_set.call(target.to_string()),
                        if target == "published" { "Publish" } else { "Unpublish" }
                    }
                }
            }
        }
    }
}
