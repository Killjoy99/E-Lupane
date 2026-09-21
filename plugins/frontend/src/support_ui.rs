//! User Support UI: FAQ browse + support tickets ("raise a ticket", my tickets).
//! Rendered for any signed-in user; staff actions (assign/status/FAQ authoring)
//! are enforced server-side.

use dioxus::prelude::*;

use crate::dashboard::UiCtx;
use crate::server_functions::{support_faq_list, support_ticket_create, support_tickets_list};
use crate::support_types::{CreateTicketInput, SupportFaqView, SupportTicketView};

/// The support panel. Reads the session token from the dashboard context.
#[component]
pub fn SupportPage() -> Element {
    let mut ctx = use_context::<UiCtx>();
    let token = (ctx.session)().map(|s| s.token.clone());

    // Load FAQ + my tickets on mount (only when signed in).
    let t_faq = token.clone();
    let t_tix = token.clone();
    let faq_res = use_resource(move || {
        let token = t_faq.clone();
        async move {
            match token {
                Some(t) => support_faq_list(t).await.unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });
    let tickets_res = use_resource(move || {
        let token = t_tix.clone();
        async move {
            match token {
                Some(t) => support_tickets_list(t).await.unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });

    // New-ticket form state.
    let mut subject = use_signal(String::new);
    let mut category = use_signal(|| "general".to_string());
    let mut priority = use_signal(|| "normal".to_string());
    let mut message: Signal<Option<String>> = use_signal(|| None);
    let mut busy = use_signal(|| false);

    // Precompute rows (outside rsx).
    let faq_ref = faq_res.read();
    let faq_items: &Option<Vec<SupportFaqView>> = &*faq_ref;
    let mut faq_rows = Vec::new();
    if let Some(items) = faq_items {
        for f in items {
            faq_rows.push(rsx! {
                div { key: "{f.id}", class: "faq-item",
                    h4 { "{f.title}" }
                    p { class: "muted", "{f.body}" }
                }
            });
        }
    }

    let tk_ref = tickets_res.read();
    let tk_items: &Option<Vec<SupportTicketView>> = &*tk_ref;
    let mut ticket_rows = Vec::new();
    if let Some(items) = tk_items {
        for t in items {
            ticket_rows.push(rsx! {
                div { key: "{t.id}", class: "ticket-item",
                    b { "{t.subject}" }
                    span { class: "muted", "  [{t.status} · {t.priority}]" }
                }
            });
        }
    }

    // Submit handler for the "raise a ticket" form.
    let submit = move |_| {
        if busy() {
            return;
        }
        let Some(token) = (ctx.session)().map(|s| s.token.clone()) else {
            message.set(Some("Please sign in to raise a ticket.".to_string()));
            return;
        };
        let s = subject();
        if s.trim().is_empty() {
            message.set(Some("Please enter a subject.".to_string()));
            return;
        }
        let cat = category();
        let pri = priority();
        busy.set(true);
        spawn(async move {
            let input = CreateTicketInput {
                subject: s.trim().to_string(),
                category: cat,
                priority: pri,
                course_id: None,
            };
            let res = support_ticket_create(token, input).await;
            busy.set(false);
            match res {
                Ok(t) => message.set(Some(format!("Ticket created: {}. Our team will get back to you.", t.subject))),
                Err(e) => message.set(Some(e.to_string())),
            }
        });
    };

    rsx! {
        div { class: "md-card support-page",
            h3 { "User Support" }
            p { class: "muted", "Find answers in the FAQ or raise a ticket and our support team will help." }

            if let Some(m) = message() {
                div { class: "auth-info", "{m}" }
            }

            div { class: "support-section",
                h4 { "FAQ / Knowledge base" }
                if faq_rows.is_empty() {
                    p { class: "muted", "No articles yet." }
                } else {
                    {faq_rows.into_iter()}
                }
            }

            div { class: "support-section",
                h4 { "Raise a ticket" }
                input { placeholder: "Subject", value: "{subject}", oninput: move |e| subject.set(e.value()) }
                div { class: "form-grid",
                    select { oninput: move |e| category.set(e.value()) }
                        option { value: "general", selected: category() == "general", "General" }
                        option { value: "login", selected: category() == "login", "Login" }
                        option { value: "courses", selected: category() == "courses", "Courses" }
                        option { value: "assignments", selected: category() == "assignments", "Assignments" }
                        option { value: "technical", selected: category() == "technical", "Technical" }
                    select { oninput: move |e| priority.set(e.value()) }
                        option { value: "low", selected: priority() == "low", "Low" }
                        option { value: "normal", selected: priority() == "normal", "Normal" }
                        option { value: "high", selected: priority() == "high", "High" }
                        option { value: "urgent", selected: priority() == "urgent", "Urgent" }
                }
                button { disabled: busy(), onclick: submit, "Submit ticket" }
            }

            div { class: "support-section",
                h4 { "My tickets" }
                if ticket_rows.is_empty() {
                    p { class: "muted", "No tickets yet." }
                } else {
                    {ticket_rows.into_iter()}
                }
            }
        }
    }
}
