//! Assignment UI components (live - backed by the assignments plugin).

use dioxus::prelude::*;

use crate::assignments_types;
use crate::components::*;
use crate::course_types;
use crate::dashboard::UiCtx;
use crate::server_functions::{
    assignments_attach, assignments_create, assignments_delete, assignments_grade,
    assignments_grade_get, assignments_list, assignments_status, assignments_submit,
    assignments_update, courses_list,
};
use crate::ui_helpers::{fmt_dt, read_file_base64};

/// Student page: list assignments, open one to submit / view status & grade.
#[component]
pub fn StudentAssignments() -> Element {
    let mut ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());
    let mut selected: Signal<Option<assignments_types::AssignmentView>> = use_signal(|| None);
    let mut refresh = use_signal(|| 0u32);
    let token_fetch = token.clone();

    let mut assignments = use_resource(move || {
        let token = token_fetch.clone();
        async move {
            let _ = refresh();
            match &token {
                Some(t) => assignments_list(t.clone()).await.unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });
    let list_ref = assignments.read();
    let list_opt: &Option<Vec<assignments_types::AssignmentView>> = &*list_ref;

    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Assignments",
                subtitle: "View briefs, submit work, resubmit, and track grades",
                action: rsx! { Button { icon: "refresh", variant: "tonal", onclick: move |_| refresh += 1, "Refresh" } }
            }

            div { class: "md-card table-wrap",
                table { class: "md-table",
                    thead { tr {
                        th { "Assignment" }
                        th { "Due" }
                        th { "Max marks" }
                        th { "Submission type" }
                        th { "Late policy" }
                        th { "Action" }
                    } }
                    tbody {
                        if let Some(list) = list_opt {
                            if list.is_empty() {
                                tr { td { colspan: "6", class: "muted", "No assignments published yet." } }
                            } else {
                                for a in list.iter().cloned() {
                                    tr {
                                        td { class: "td-strong", "{a.title}" }
                                        td { "{fmt_dt(a.deadline)}" }
                                        td { "{a.max_marks}" }
                                        td { "{a.submission_type}" }
                                        td { "{a.late_submission_policy}" }
                                        td {
                                            Button {
                                                variant: "tonal", icon: "open_in_new",
                                                onclick: move |_| selected.set(Some(a.clone())),
                                                "Open",
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            tr { td { colspan: "6", class: "muted", "Loading…" } }
                        }
                    }
                }
            }

            if let Some(a) = selected() {
                if let Some(t) = token.clone() {
                    StudentAssignmentDetail { assignment: a, token: t, on_close: move |_| selected.set(None) }
                }
            }
        }
    }
}
/// Student detail: instructions + status/grade + submission form.
#[component]
pub fn StudentAssignmentDetail(
    assignment: assignments_types::AssignmentView,
    token: String,
    on_close: EventHandler<()>,
) -> Element {
    let token_fetch = token.clone();
    let mut data = use_resource(move || {
        let token = token_fetch.clone();
        async move {
            let status = assignments_status(token.clone(), assignment.id).await.ok();
            let mut grade = None;
            if let Some(sv) = &status {
                if let Ok(sub) =
                    serde_json::from_value::<assignments_types::SubmissionView>(sv.clone())
                {
                    if sub.graded {
                        grade = assignments_grade_get(token.clone(), sub.id).await.ok();
                    }
                }
            }
            (status, grade)
        }
    });

    let due = fmt_dt(assignment.deadline);
    let max_marks = assignment.max_marks;
    let attempts = assignment.allowed_attempts;
    let data_ref = data.read();
    let data_opt: &Option<(Option<serde_json::Value>, Option<assignments_types::GradeView>)> = &*data_ref;

    let status_row = match data_opt {
        Some((Some(sv), grade_opt)) => {
            match serde_json::from_value::<assignments_types::SubmissionView>(sv.clone()) {
                Ok(sub) => {
                    let submitted = fmt_dt(Some(sub.submitted_at));
                    let tone = if sub.is_late {
                        "flagged"
                    } else if sub.graded {
                        "graded"
                    } else {
                        "submitted"
                    };
                    let grade_cell = if sub.graded {
                        match grade_opt {
                            Some(g) => format!("{:.1} / {}", g.marks, max_marks),
                            None => "graded".to_string(),
                        }
                    } else {
                        "—".to_string()
                    };
                    rsx! {
                        tr {
                            td { "{submitted}" }
                            td { "#{sub.attempt}" }
                            td { StatusChip { status: tone.to_string() } }
                            td { "{grade_cell}" }
                        }
                    }
                }
                Err(_) => rsx! { tr { td { colspan: "4", class: "muted", "No submission yet." } } },
            }
        }
        _ => rsx! { tr { td { colspan: "4", class: "muted", "No submission yet." } } },
    };

    rsx! {
        div { class: "md-card assignment-detail",
            div { class: "assignment-detail-head",
                div {
                    h3 { "{assignment.title}" }
                    p { class: "muted", "Due {due} · {max_marks} marks · {attempts} attempt(s) allowed" }
                }
                Button { variant: "text", icon: "close", onclick: move |_| on_close.call(()) }
            }
            div { class: "assignment-instructions",
                p { "Instructions" }
                div { class: "muted", "{assignment.instructions}" }
            }
            div { class: "md-card table-wrap", style: "margin-top: 12px",
                table { class: "md-table",
                    thead { tr { th { "Latest submission" } th { "Attempt" } th { "Status" } th { "Grade" } } }
                    tbody {
                        {status_row}
                    }
                }
            }
            StudentSubmissionForm { assignment: assignment, token: token }
        }
    }
}
/// The student submission form (text / link / file depending on the type).
#[component]
pub fn StudentSubmissionForm(assignment: assignments_types::AssignmentView, token: String) -> Element {
    let mut text = use_signal(String::new);
    let mut link_url = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut message: Signal<Option<String>> = use_signal(|| None);
    // Pre-clone for the submit closure so the originals stay usable below.
    let submit_token = token.clone();
    let submit_assignment = assignment.clone();

    rsx! {
        div { class: "submission-form",
            if let Some(m) = message() {
                div { class: "auth-error", "{m}" }
            }
            if assignment.submission_type == "text" || assignment.submission_type == "any" {
                textarea {
                    placeholder: "Write your answer…",
                    value: "{text}",
                    oninput: move |e| text.set(e.value()),
                }
            }
            if assignment.submission_type == "link" || assignment.submission_type == "any" {
                input {
                    placeholder: "https://… (link submission)",
                    value: "{link_url}",
                    oninput: move |e| link_url.set(e.value()),
                }
            }
            if assignment.submission_type == "file" || assignment.submission_type == "any" {
                input { id: "student-file-input", r#type: "file" }
            }
            div { class: "submission-actions",
                Button {
                    icon: "upload",
                    disabled: busy(),
                    onclick: move |_| {
                        spawn(run_student_submit(
                            submit_token.clone(),
                            submit_assignment.clone(),
                            text,
                            link_url,
                            busy,
                            message,
                        ));
                    },
                    if busy() { "Submitting…" } else { "Submit" }
                }
                if assignment.submission_type == "file" || assignment.submission_type == "any" {
                    span { class: "muted", style: "font-size: 12px", "Max 25 MiB per file." }
                }
            }
        }
    }
}

/// Execute a student submission (reads the file input + form signals).
async fn run_student_submit(
    token: String,
    assignment: assignments_types::AssignmentView,
    mut text: Signal<String>,
    mut link_url: Signal<String>,
    mut busy: Signal<bool>,
    mut message: Signal<Option<String>>,
) {
    busy.set(true);
    message.set(None);
    let file = read_file_base64("student-file-input").await;
    let files = file.map(|f| vec![f]).unwrap_or_default();
    let links = if link_url().trim().is_empty() {
        Vec::new()
    } else {
        vec![assignments_types::LinkInput {
            url: link_url().trim().to_string(),
            title: String::new(),
        }]
    };
    let text_val = if text().trim().is_empty() {
        None
    } else {
        Some(text().trim().to_string())
    };
    match assignments_submit(token, assignment.id, text_val, files, links).await {
        Ok(sv) => {
            message.set(Some(format!(
                "Submitted successfully — attempt #{}.",
                sv.attempt
            )));
            text.set(String::new());
            link_url.set(String::new());
        }
        Err(e) => message.set(Some(e.to_string())),
    }
    busy.set(false);
}




/// Create/edit assignment form (used by lecturers and admins).
#[component]
pub fn AssignmentEditor(
    token: String,
    assignment: Option<assignments_types::AssignmentView>,
    on_done: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let a = assignment.clone();
    let mut title = use_signal(|| a.as_ref().map(|x| x.title.clone()).unwrap_or_default());
    let mut instructions = use_signal(|| a.as_ref().map(|x| x.instructions.clone()).unwrap_or_default());
    let mut deadline = use_signal(|| a.as_ref().and_then(|x| x.deadline).map(|d| d.format("%Y-%m-%dT%H:%M").to_string()).unwrap_or_default());
    let mut max_marks = use_signal(|| a.as_ref().map(|x| x.max_marks.to_string()).unwrap_or_else(|| "100".into()));
    let mut attempts = use_signal(|| a.as_ref().map(|x| x.allowed_attempts.to_string()).unwrap_or_else(|| "1".into()));
    let mut stype = use_signal(|| a.as_ref().map(|x| x.submission_type.clone()).unwrap_or_else(|| "any".into()));
    let mut policy = use_signal(|| a.as_ref().map(|x| x.late_submission_policy.clone()).unwrap_or_else(|| "allowed_with_penalty".into()));
    let mut penalty = use_signal(|| a.as_ref().map(|x| x.late_penalty_percent.to_string()).unwrap_or_else(|| "0".into()));
    let mut busy = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut course_id = use_signal(|| assignment.as_ref().and_then(|x| x.course_id));
    let editor_title: &'static str = if assignment.is_some() { "Edit assignment" } else { "New assignment" };

    // Courses the instructor manages (for linking assignments to courses).
    let token_fetch = token.clone();
    let mut courses = use_resource(move || {
        let token = token_fetch.clone();
        async move { courses_list(token, None, true, None).await.unwrap_or_default() }
    });
    let courses_ref = courses.read();
    let courses_opt: &Option<Vec<course_types::CourseView>> = &*courses_ref;
    // Precompute (id, label) pairs outside the rsx (rsx! `let` cannot call methods).
    let course_options: Vec<(uuid::Uuid, String)> = courses_opt
        .as_ref()
        .map(|v| {
            v.iter()
                .map(|c| (c.id, format!("{} — {}", c.code, c.title)))
                .collect()
        })
        .unwrap_or_default();

    rsx! {
        div { class: "md-card assignment-editor",
            SectionTitle { title: editor_title }
            if let Some(e) = error() {
                div { class: "auth-error", "{e}" }
            }
            div { class: "form-grid",
                div { class: "field-full", label { "Title" } input { value: "{title}", oninput: move |e| title.set(e.value()) } }
                div { class: "field-full", label { "Instructions" } textarea { rows: "3", value: "{instructions}", oninput: move |e| instructions.set(e.value()) } }
                div { label { "Deadline (optional)" } input { r#type: "datetime-local", value: "{deadline}", oninput: move |e| deadline.set(e.value()) } }
                div { label { "Max marks" } input { r#type: "number", value: "{max_marks}", oninput: move |e| max_marks.set(e.value()) } }
                div { label { "Allowed attempts" } input { r#type: "number", value: "{attempts}", oninput: move |e| attempts.set(e.value()) } }
                div {
                    label { "Submission type" }
                    select { oninput: move |e| stype.set(e.value()) }
                        option { value: "any", selected: stype() == "any", "Any" }
                        option { value: "file", selected: stype() == "file", "File" }
                        option { value: "text", selected: stype() == "text", "Text" }
                        option { value: "link", selected: stype() == "link", "Link" }
                }
                div {
                    label { "Late policy" }
                    select { oninput: move |e| policy.set(e.value()) }
                        option { value: "disallowed", selected: policy() == "disallowed", "Disallowed" }
                        option { value: "allowed", selected: policy() == "allowed", "Allowed" }
                        option { value: "allowed_with_penalty", selected: policy() == "allowed_with_penalty", "Allowed with penalty" }
                }
                div { label { "Late penalty (%)" } input { r#type: "number", value: "{penalty}", oninput: move |e| penalty.set(e.value()) } }
                div {
                    label { "Course (optional)" }
                    select { oninput: move |e| {
                        if e.value().is_empty() {
                            course_id.set(None);
                        } else {
                            if let Ok(id) = uuid::Uuid::parse_str(&e.value()) { course_id.set(Some(id)); }
                        }
                    } }
                        option { value: "", selected: course_id().is_none(), "— No course —" }
                        for (cuid, label) in course_options.iter() {
                            option { value: "{cuid}", selected: course_id() == Some(*cuid), "{label}" }
                        }
                }
            }
            div { class: "submission-actions",
                Button {
                    icon: "save",
                    disabled: busy(),
                    onclick: move |_| {
                        let token = token.clone();
                        let existing_id = assignment.as_ref().map(|x| x.id);
                        spawn(async move {
                            busy.set(true);
                            error.set(None);
                            let deadline_dt = if deadline().trim().is_empty() {
                                None
                            } else {
                                chrono::DateTime::parse_from_str(&format!("{}:00Z", deadline().trim()), "%Y-%m-%dT%H:%M:%SZ")
                                    .ok()
                                    .map(|d| d.with_timezone(&chrono::Utc))
                            };
                            let input = assignments_types::AssignmentInput {
                                title: title().trim().to_string(),
                                instructions: instructions().trim().to_string(),
                                deadline: deadline_dt,
                                max_marks: max_marks().parse().unwrap_or(100.0),
                                allowed_attempts: attempts().parse().unwrap_or(1),
                                submission_type: stype(),
                                late_submission_policy: policy(),
                                late_penalty_percent: penalty().parse().unwrap_or(0.0),
                                course_id: course_id(),
                            };
                            let result = match existing_id {
                                Some(id) => assignments_update(token, id, input).await,
                                None => assignments_create(token, input).await,
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


/// Instructor view of a single assignment's submissions, with grading.
#[component]
pub fn SubmissionsPanel(token: String, assignment: assignments_types::AssignmentView, on_close: EventHandler<()>) -> Element {
    let token_fetch = token.clone();
    let mut submissions = use_resource(move || {
        let token = token_fetch.clone();
        async move {
            assignments_status(token, assignment.id)
                .await
                .ok()
                .and_then(|v| serde_json::from_value::<Vec<assignments_types::SubmissionView>>(v).ok())
                .unwrap_or_default()
        }
    });

    let subs_ref = submissions.read();
    let subs_opt: &Option<Vec<assignments_types::SubmissionView>> = &*subs_ref;

    rsx! {
        div { class: "md-card table-wrap", style: "margin-top: 16px",
            div { class: "assignment-detail-head",
                div { h3 { "Submissions — {assignment.title}" } }
                Button { variant: "text", icon: "close", onclick: move |_| on_close.call(()) }
            }
            table { class: "md-table",
                thead { tr {
                    th { "Student" }
                    th { "Attempt" }
                    th { "Submitted" }
                    th { "Late" }
                    th { "Text" }
                    th { "Files / Links" }
                    th { "Grade" }
                } }
                tbody {
                    if let Some(subs) = subs_opt {
                        for sub in subs.iter().cloned() {
                            SubmissionRow { token: token.clone(), assignment: assignment.clone(), submission: sub }
                        }
                        if subs.is_empty() {
                            tr { td { colspan: "7", class: "muted", "No submissions yet." } }
                        }
                    } else {
                        tr { td { colspan: "7", class: "muted", "Loading…" } }
                    }
                }
            }
        }
    }
}

/// One submission row with a grading form.
#[component]
pub fn SubmissionRow(
    token: String,
    assignment: assignments_types::AssignmentView,
    submission: assignments_types::SubmissionView,
) -> Element {
    let mut marks = use_signal(String::new);
    let mut comments = use_signal(String::new);
    let mut feedback = use_signal(String::new);
    let mut done: Signal<Option<String>> = use_signal(|| None);
    let submitted = fmt_dt(Some(submission.submitted_at));
    let counts = format!("{} file(s) · {} link(s)", submission.files.len(), submission.links.len());
    let submission_id = submission.id;
    let graded = submission.graded;
    let max_marks = assignment.max_marks;

    rsx! {
        tr {
            td { "{submission.student_id}" }
            td { "#{submission.attempt}" }
            td { "{submitted}" }
            td { if submission.is_late { StatusChip { status: "flagged".to_string(), label: Some("late".to_string()) } } else { "—" } }
            td { if let Some(t) = submission.text.as_ref() { "{t}" } else { "—" } }
            td { "{counts}" }
            td {
                div { class: "grade-inline",
                    input {
                        r#type: "number",
                        placeholder: "marks",
                        style: "width: 70px",
                        value: "{marks}",
                        oninput: move |e| marks.set(e.value()),
                    }
                    Button {
                        variant: "tonal", icon: "fact_check",
                        onclick: move |_| {
                            let token = token.clone();
                            let mut marks = marks;
                            let mut comments = comments;
                            let mut feedback = feedback;
                            let mut done = done;
                            spawn(async move {
                                let m = marks().parse().unwrap_or(0.0);
                                match assignments_grade(token, submission_id, m, comments(), feedback()).await {
                                    Ok(g) => done.set(Some(format!("Graded: {:.1}", g.marks))),
                                    Err(e) => done.set(Some(e.to_string())),
                                }
                            });
                        },
                        "Grade",
                    }
                    input { placeholder: "comments", value: "{comments}", oninput: move |e| comments.set(e.value()) }
                    input { placeholder: "feedback", value: "{feedback}", oninput: move |e| feedback.set(e.value()) }
                }
                if let Some(d) = done() {
                    div { class: "muted", style: "font-size: 12px", "{d}" }
                }
                if graded {
                    div { class: "muted", style: "font-size: 12px", "Previously graded (max {max_marks})." }
                }
            }
        }
    }
}


/// Shared instructor assignment manager: list, create/edit, delete, attach
/// resources, and review submissions. Used by lecturers and admins.
#[component]
pub fn AssignmentManager(subtitle: &'static str) -> Element {
    let mut ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());
    let mut editing: Signal<Option<assignments_types::AssignmentView>> = use_signal(|| None);
    let mut show_new = use_signal(|| false);
    let mut submissions_for: Signal<Option<uuid::Uuid>> = use_signal(|| None);
    let mut attach_for: Signal<Option<uuid::Uuid>> = use_signal(|| None);
    let mut message: Signal<Option<String>> = use_signal(|| None);
    let mut refresh = use_signal(|| 0u32);
    let token_fetch = token.clone();

    let mut assignments = use_resource(move || {
        let token = token_fetch.clone();
        async move {
            let _ = refresh();
            match &token {
                Some(t) => assignments_list(t.clone()).await.unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });

    let list_ref = assignments.read();
    let list_opt: &Option<Vec<assignments_types::AssignmentView>> = &*list_ref;
    let submissions_assignment = match submissions_for() {
        Some(aid) => list_opt
            .as_ref()
            .and_then(|items| items.iter().find(|a| a.id == aid))
            .cloned(),
        None => None,
    };

    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Assignments",
                subtitle: subtitle,
                action: rsx! {
                    Button { icon: "add", onclick: move |_| { editing.set(None); show_new.set(true); }, "New assignment" }
                    Button { variant: "tonal", icon: "refresh", onclick: move |_| refresh += 1, "Refresh" }
                }
            }
            if let Some(m) = message() {
                div { class: "auth-error", "{m}" }
            }
            if show_new() {
                if let Some(t) = token.clone() {
                    AssignmentEditor { token: t, assignment: None, on_done: move |_| { show_new.set(false); refresh += 1; }, on_cancel: move |_| show_new.set(false) }
                }
            }
            if let Some(edit) = editing() {
                if let Some(t) = token.clone() {
                    AssignmentEditor { token: t, assignment: Some(edit), on_done: move |_| { editing.set(None); refresh += 1; }, on_cancel: move |_| editing.set(None) }
                }
            }
            if let Some(a) = submissions_assignment.clone() {
                if let Some(t) = token.clone() {
                    SubmissionsPanel { token: t, assignment: a, on_close: move |_| submissions_for.set(None) }
                }
            }
            if let Some(aid) = attach_for() {
                if let Some(t) = token.clone() {
                    AttachResourceForm { token: t, assignment_id: aid, on_done: move |_| { attach_for.set(None); refresh += 1; } }
                }
            }

            div { class: "md-card table-wrap",
                table { class: "md-table",
                    thead { tr { th { "Title" } th { "Due" } th { "Max" } th { "Type" } th { "Attempts" } th { "Actions" } } }
                    tbody {
                        if let Some(list) = list_opt {
                            if list.is_empty() {
                                tr { td { colspan: "6", class: "muted", "No assignments yet. Create one to get started." } }
                            } else {
                                for a in list.iter().cloned() {
                                    tr {
                                        td { class: "td-strong", "{a.title}" }
                                        td { "{fmt_dt(a.deadline)}" }
                                        td { "{a.max_marks}" }
                                        td { "{a.submission_type}" }
                                        td { "{a.allowed_attempts}" }
                                        td { class: "row-actions",
                                            Button { variant: "tonal", icon: "fact_check", onclick: { let a = a.clone(); move |_| submissions_for.set(Some(a.id)) }, "Submissions" }
                                            Button { variant: "tonal", icon: "edit", onclick: { let a = a.clone(); move |_| editing.set(Some(a.clone())) }, "Edit" }
                                            Button { variant: "tonal", icon: "attach_file", onclick: { let a = a.clone(); move |_| attach_for.set(Some(a.id)) }, "Attach" }
                                            Button { class: "md-btn-danger", variant: "text", icon: "delete", onclick: {
                                                let token = token.clone();
                                                let a = a.clone();
                                                move |_| {
                                                    let id = a.id;
                                                    let token = token.clone();
                                                    spawn(async move {
                                                        match token {
                                                            Some(t) => match assignments_delete(t, id).await {
                                                                Ok(_) => { message.set(Some("Assignment deleted.".into())); refresh += 1; }
                                                                Err(e) => message.set(Some(e.to_string())),
                                                            },
                                                            None => message.set(Some("Not signed in.".into())),
                                                        }
                                                    });
                                                }
                                            }, "Delete" }
                                        }
                                    }
                                }
                            }
                        } else {
                            tr { td { colspan: "6", class: "muted", "Loading…" } }
                        }
                    }
                }
            }
        }
    }
}

/// Attach a resource file to an assignment.
#[component]
pub fn AttachResourceForm(token: String, assignment_id: uuid::Uuid, on_done: EventHandler<()>) -> Element {
    let mut busy = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div { class: "md-card assignment-editor",
            SectionTitle { title: "Attach resource" }
            if let Some(e) = error() {
                div { class: "auth-error", "{e}" }
            }
            input { id: "lecturer-file-input", r#type: "file" }
            div { class: "submission-actions",
                Button {
                    icon: "upload",
                    disabled: busy(),
                    onclick: move |_| {
                        let token = token.clone();
                        let aid = assignment_id;
                        spawn(async move {
                            busy.set(true);
                            error.set(None);
                            match read_file_base64("lecturer-file-input").await {
                                Some(f) => match assignments_attach(token, aid, f.name, f.mime_type, f.content_base64).await {
                                    Ok(_) => on_done.call(()),
                                    Err(e) => error.set(Some(e.to_string())),
                                },
                                None => error.set(Some("Choose a file first.".into())),
                            }
                            busy.set(false);
                        });
                    },
                    if busy() { "Uploading…" } else { "Upload" }
                }
                Button { variant: "text", icon: "close", onclick: move |_| on_done.call(()), "Cancel" }
            }
        }
    }
}

/// Lecturer assignments dashboard.
#[component]
pub fn LecturerAssignments() -> Element {
    rsx! { AssignmentManager { subtitle: "Create, edit and grade assignments for your courses" } }
}

/// Admin assignments dashboard (admins manage assignments like instructors).
#[component]
pub fn AdminAssignments() -> Element {
    rsx! { AssignmentManager { subtitle: "Institution-wide assignment management" } }
}


