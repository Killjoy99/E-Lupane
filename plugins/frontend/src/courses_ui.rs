//! Course UI components (live - backed by the course_management plugin).

use dioxus::prelude::*;

use crate::components::*;
use crate::course_types;
use crate::dashboard::UiCtx;
use crate::server_functions::{
    courses_archive, courses_create, courses_delete, courses_duplicate, courses_list,
    courses_publish, courses_update,
};

/// Student course catalogue (live — backed by the course_management plugin).
#[component]
pub fn StudentCourses() -> Element {
    let mut ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());
    let token_fetch = token.clone();
    let mut refresh = use_signal(|| 0u32);
    let mut selected: Signal<Option<course_types::CourseView>> = use_signal(|| None);

    let mut courses = use_resource(move || {
        let token = token_fetch.clone();
        async move {
            let _ = refresh();
            match token {
                Some(t) => courses_list(t, Some("published".to_string()), false, None)
                    .await
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });
    let list_ref = courses.read();
    let list_opt: &Option<Vec<course_types::CourseView>> = &*list_ref;

    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Course catalogue",
                subtitle: "Published courses available for enrolment",
                action: rsx! { Button { variant: "tonal", icon: "refresh", onclick: move |_| refresh += 1, "Refresh" } }
            }
            if let Some(list) = list_opt {
                if list.is_empty() {
                    div { class: "md-card placeholder",
                        span { class: "material-symbols-outlined placeholder-icon", "school" }
                        h3 { "No courses published yet" }
                        p { "Courses appear here once their instructors publish them." }
                    }
                } else {
                    div { class: "grid-3",
                        for c in list.iter().cloned() {
                            CourseCatalogCard {
                                course: c.clone(),
                                on_open: { let c = c.clone(); move |_| selected.set(Some(c.clone())) },
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
            if let Some(c) = selected() {
                CourseDetailPanel { course: c, on_close: { let mut selected = selected; move |_| selected.set(None) } }
            }
        }
    }
}

/// A compact catalogue card for a published course.
#[component]
pub fn CourseCatalogCard(course: course_types::CourseView, on_open: EventHandler<()>) -> Element {
    let code = course.code.clone();
    let title = course.title.clone();
    let short = course.short_description.clone();
    let subject = course.subject.clone();
    let difficulty = course.difficulty.clone();
    let duration = course.duration.clone();

    rsx! {
        div { class: "md-card course-card",
            div { class: "course-card-head",
                span { class: "course-icon course-icon-teal",
                    span { class: "material-symbols-outlined", "menu_book" }
                }
                span { class: "course-code", "{code}" }
            }
            h3 { "{title}" }
            div { class: "course-meta", "{subject}" }
            p { class: "muted", style: "font-size: 13px", "{short}" }
            div { class: "course-foot",
                span { class: "course-students", "{difficulty} · {duration}" }
                Button { variant: "tonal", icon: "visibility", onclick: move |_| on_open.call(()), "View" }
            }
        }
    }
}

/// Read-only detail panel for a course (student preview).
#[component]
pub fn CourseDetailPanel(course: course_types::CourseView, on_close: EventHandler<()>) -> Element {
    let objectives = course.learning_objectives.join(" · ");
    let prereqs = course.prerequisites.join(" · ");
    let audience = course.target_audience.join(" · ");
    let tags = course.tags.join(", ");
    let status = course_types::course_status_label(&course.status);

    rsx! {
        div { class: "md-card assignment-detail",
            div { class: "assignment-detail-head",
                div {
                    h3 { "{course.code} — {course.title}" }
                    p { class: "muted", "{status} · {course.subject} · {course.difficulty} · {course.duration}" }
                }
                Button { variant: "text", icon: "close", onclick: move |_| on_close.call(()) }
            }
            div { class: "assignment-instructions",
                p { "Description" }
                div { class: "muted", "{course.description}" }
            }
            div { class: "form-grid",
                div { label { "Category" } div { class: "muted", "{course.category}" } }
                div { label { "Level" } div { class: "muted", "{course.level}" } }
                div { label { "Language" } div { class: "muted", "{course.language}" } }
                div { label { "Learning objectives" } div { class: "muted", "{objectives}" } }
                div { label { "Prerequisites" } div { class: "muted", "{prereqs}" } }
                div { label { "Target audience" } div { class: "muted", "{audience}" } }
                div { class: "field-full", label { "Tags" } div { class: "muted", "{tags}" } }
            }
        }
    }
}




/// Instructor/admin course management (live — backed by the course_management
/// plugin). Create, edit, preview, publish/unpublish, archive, duplicate, delete.
#[component]
pub fn CourseManager() -> Element {
    let mut ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());
    let token_fetch = token.clone();
    let mut refresh = use_signal(|| 0u32);
    let mut editing: Signal<Option<course_types::CourseView>> = use_signal(|| None);
    let mut show_new = use_signal(|| false);
    let mut preview: Signal<Option<course_types::CourseView>> = use_signal(|| None);

    let mut courses = use_resource(move || {
        let token = token_fetch.clone();
        async move {
            let _ = refresh();
            match token {
                Some(t) => courses_list(t, None, true, None).await.unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });
    let list_ref = courses.read();
    let list_opt: &Option<Vec<course_types::CourseView>> = &*list_ref;

    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Course management",
                subtitle: "Create, review, publish and manage courses",
                action: rsx! {
                    Button { icon: "add", onclick: move |_| { editing.set(None); show_new.set(true); }, "New course" }
                    Button { variant: "tonal", icon: "refresh", onclick: move |_| refresh += 1, "Refresh" }
                }
            }
            if show_new() {
                if let Some(t) = token.clone() {
                    CourseEditor { token: t, course: None, on_done: move |_| { show_new.set(false); refresh += 1; }, on_cancel: move |_| show_new.set(false) }
                }
            }
            if let Some(edit) = editing() {
                if let Some(t) = token.clone() {
                    CourseEditor { token: t, course: Some(edit), on_done: move |_| { editing.set(None); refresh += 1; }, on_cancel: move |_| editing.set(None) }
                }
            }
            if let Some(c) = preview() {
                CourseDetailPanel { course: c, on_close: { let mut preview = preview; move |_| preview.set(None) } }
            }

            div { class: "md-card table-wrap",
                table { class: "md-table",
                    thead { tr {
                        th { "Code" } th { "Title" } th { "Status" } th { "Difficulty" } th { "Actions" }
                    } }
                    tbody {
                        if let Some(list) = list_opt {
                            if list.is_empty() {
                                tr { td { colspan: "5", class: "muted", "No courses yet. Create one to get started." } }
                            } else {
                                for c in list.iter().cloned() {
                                    if let Some(t) = token.clone() {
                                        CourseRow {
                                            token: t,
                                            course: c.clone(),
                                            on_edit: { let c = c.clone(); move |_| editing.set(Some(c.clone())) },
                                            on_preview: { let c = c.clone(); move |_| preview.set(Some(c.clone())) },
                                            on_refresh: { let mut refresh = refresh; move |_| refresh += 1 },
                                        }
                                    }
                                }
                            }
                        } else {
                            tr { td { colspan: "5", class: "muted", "Loading…" } }
                        }
                    }
                }
            }
        }
    }
}


/// One course row with lifecycle + management actions.
#[component]
pub fn CourseRow(
    token: String,
    course: course_types::CourseView,
    on_edit: EventHandler<()>,
    on_preview: EventHandler<()>,
    on_refresh: EventHandler<()>,
) -> Element {
    let status = course.status.clone();
    let tone = course_types::course_status_tone(&status);
    let label = course_types::course_status_label(&status);

    rsx! {
        tr {
            td { class: "td-strong", "{course.code}" }
            td { "{course.title}" }
            td { StatusChip { status: tone, label: Some(label) } }
            td { "{course.difficulty}" }
            td { class: "row-actions",
                Button { variant: "tonal", icon: "edit", onclick: move |_| on_edit.call(()), "Edit" }
                Button { variant: "tonal", icon: "visibility", onclick: move |_| on_preview.call(()), "Preview" }
                if status != "published" {
                    Button { variant: "tonal", icon: "publish", onclick: {
                        let token = token.clone();
                        let id = course.id;
                        move |_| { let token = token.clone(); spawn(async move { let _ = courses_publish(token, id).await; on_refresh.call(()); }); }
                    }, "Publish" }
                }
                if status != "archived" {
                    Button { variant: "tonal", icon: "archive", onclick: {
                        let token = token.clone();
                        let id = course.id;
                        move |_| { let token = token.clone(); spawn(async move { let _ = courses_archive(token, id).await; on_refresh.call(()); }); }
                    }, "Archive" }
                }
                Button { variant: "tonal", icon: "content_copy", onclick: {
                    let token = token.clone();
                    let id = course.id;
                    move |_| { let token = token.clone(); spawn(async move { let _ = courses_duplicate(token, id).await; on_refresh.call(()); }); }
                }, "Duplicate" }
                Button { class: "md-btn-danger", variant: "text", icon: "delete", onclick: {
                    let token = token.clone();
                    let id = course.id;
                    move |_| { let token = token.clone(); spawn(async move { let _ = courses_delete(token, id).await; on_refresh.call(()); }); }
                }, "Delete" }
            }
        }
    }
}

/// Create/edit course form.
#[component]
pub fn CourseEditor(
    token: String,
    course: Option<course_types::CourseView>,
    on_done: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let c = course.clone();
    let mut code = use_signal(|| c.as_ref().map(|x| x.code.clone()).unwrap_or_default());
    let mut title = use_signal(|| c.as_ref().map(|x| x.title.clone()).unwrap_or_default());
    let mut short = use_signal(|| c.as_ref().map(|x| x.short_description.clone()).unwrap_or_default());
    let mut desc = use_signal(|| c.as_ref().map(|x| x.description.clone()).unwrap_or_default());
    let mut category = use_signal(|| c.as_ref().map(|x| x.category.clone()).unwrap_or_default());
    let mut subject = use_signal(|| c.as_ref().map(|x| x.subject.clone()).unwrap_or_default());
    let mut level = use_signal(|| c.as_ref().map(|x| x.level.clone()).unwrap_or_default());
    let mut language = use_signal(|| c.as_ref().map(|x| x.language.clone()).unwrap_or_else(|| "en".into()));
    let mut difficulty = use_signal(|| c.as_ref().map(|x| x.difficulty.clone()).unwrap_or_else(|| "beginner".into()));
    let mut duration = use_signal(|| c.as_ref().map(|x| x.duration.clone()).unwrap_or_default());
    let mut objectives = use_signal(|| c.as_ref().map(|x| x.learning_objectives.join("\n")).unwrap_or_default());
    let mut audience = use_signal(|| c.as_ref().map(|x| x.target_audience.join("\n")).unwrap_or_default());
    let mut prereqs = use_signal(|| c.as_ref().map(|x| x.prerequisites.join("\n")).unwrap_or_default());
    let mut tags = use_signal(|| c.as_ref().map(|x| x.tags.join(", ")).unwrap_or_default());
    let mut busy = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let editor_title: &'static str = if course.is_some() { "Edit course" } else { "New course" };

    rsx! {
        div { class: "md-card assignment-editor",
            SectionTitle { title: editor_title }
            if let Some(e) = error() {
                div { class: "auth-error", "{e}" }
            }
            div { class: "form-grid",
                div { label { "Code" } input { placeholder: "CS101", value: "{code}", oninput: move |e| code.set(e.value()) } }
                div { label { "Title" } input { placeholder: "Intro to Programming", value: "{title}", oninput: move |e| title.set(e.value()) } }
                div { class: "field-full", label { "Short description" } input { placeholder: "One-line summary", value: "{short}", oninput: move |e| short.set(e.value()) } }
                div { class: "field-full", label { "Description" } textarea { rows: "3", placeholder: "Full course description", value: "{desc}", oninput: move |e| desc.set(e.value()) } }
                div { label { "Category" } input { placeholder: "Computer Science", value: "{category}", oninput: move |e| category.set(e.value()) } }
                div { label { "Subject" } input { placeholder: "Programming", value: "{subject}", oninput: move |e| subject.set(e.value()) } }
                div { label { "Level" } input { placeholder: "Undergraduate", value: "{level}", oninput: move |e| level.set(e.value()) } }
                div { label { "Language" } input { placeholder: "en", value: "{language}", oninput: move |e| language.set(e.value()) } }
                div {
                    label { "Difficulty" }
                    select { oninput: move |e| difficulty.set(e.value()) }
                        option { value: "beginner", selected: difficulty() == "beginner", "Beginner" }
                        option { value: "intermediate", selected: difficulty() == "intermediate", "Intermediate" }
                        option { value: "advanced", selected: difficulty() == "advanced", "Advanced" }
                        option { value: "expert", selected: difficulty() == "expert", "Expert" }
                }
                div { label { "Duration" } input { placeholder: "8 weeks", value: "{duration}", oninput: move |e| duration.set(e.value()) } }
                div { class: "field-full", label { "Learning objectives (one per line)" } textarea { rows: "3", value: "{objectives}", oninput: move |e| objectives.set(e.value()) } }
                div { class: "field-full", label { "Target audience (one per line)" } textarea { rows: "2", value: "{audience}", oninput: move |e| audience.set(e.value()) } }
                div { class: "field-full", label { "Prerequisites (one per line)" } textarea { rows: "2", value: "{prereqs}", oninput: move |e| prereqs.set(e.value()) } }
                div { class: "field-full", label { "Tags (comma separated)" } input { placeholder: "python, intro, programming", value: "{tags}", oninput: move |e| tags.set(e.value()) } }
            }
            div { class: "submission-actions",
                Button {
                    icon: "save",
                    disabled: busy(),
                    onclick: move |_| {
                        let token = token.clone();
                        let existing_id = course.as_ref().map(|x| x.id);
                        let mut busy = busy;
                        let mut error = error;
                        spawn(async move {
                            busy.set(true);
                            error.set(None);
                            let split_lines = |s: &str| s.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect::<Vec<_>>();
                            let split_tags = |s: &str| s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect::<Vec<_>>();
                            let input = course_types::CourseInput {
                                code: code().trim().to_string(),
                                title: title().trim().to_string(),
                                short_description: short().trim().to_string(),
                                description: desc().trim().to_string(),
                                thumbnail: String::new(),
                                banner: String::new(),
                                category: category().trim().to_string(),
                                subject: subject().trim().to_string(),
                                level: level().trim().to_string(),
                                language: language().trim().to_string(),
                                difficulty: difficulty(),
                                duration: duration().trim().to_string(),
                                learning_objectives: split_lines(&objectives()),
                                target_audience: split_lines(&audience()),
                                prerequisites: split_lines(&prereqs()),
                                tags: split_tags(&tags()),
                            };
                            let result = match existing_id {
                                Some(id) => courses_update(token, id, input).await,
                                None => courses_create(token, input).await,
                            };
                            match result {
                                Ok(_) => on_done.call(()),
                                Err(e) => error.set(Some(e.to_string())),
                            }
                            busy.set(false);
                        });
                    },
                    "Save"
                }
                Button { variant: "text", icon: "close", onclick: move |_| on_cancel.call(()), "Cancel" }
            }
        }
    }
}

