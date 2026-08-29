//! Material Design 3 UI components (light theme) built on Dioxus.
//!
//! These are the building blocks of the E-Lupane dashboard: icons, buttons,
//! chips, cards, progress bars, avatars, tables, etc. They follow the Material
//! 3 design tokens defined in `assets/main.css`.

use dioxus::prelude::*;

/// Material Symbols icon (filled vs outlined via `class`).
pub fn Icon(name: &'static str, class: Option<&'static str>) -> Element {
    let class = class.unwrap_or("");
    rsx! {
        span {
            class: format!("material-symbols-outlined {class}"),
            "aria-hidden": "true",
            "{name}"
        }
    }
}

/// Round avatar showing initials, tinted by tone (teal/indigo/green/...).
#[component]
pub fn Avatar(initials: &'static str, tone: Option<&'static str>) -> Element {
    let tone = tone.unwrap_or("teal");
    rsx! {
        div { class: format!("md-avatar md-avatar-{tone}"), "{initials}" }
    }
}

/// Material button. Variants: `filled` (default), `tonal`, `outlined`, `text`.
#[component]
pub fn Button(
    children: Element,
    onclick: Option<EventHandler<MouseEvent>>,
    variant: Option<&'static str>,
    icon: Option<&'static str>,
    class: Option<&'static str>,
) -> Element {
    let variant = variant.unwrap_or("filled");
    let class = class.unwrap_or("");
    rsx! {
        button {
            class: format!("md-btn md-btn-{variant} {class}"),
            onclick: move |e| {
                if let Some(handler) = onclick {
                    handler.call(e);
                }
            },
            if let Some(icon) = icon {
                span { class: "material-symbols-outlined md-btn-icon", "{icon}" }
            }
            span { class: "md-btn-label", {children} }
        }
    }
}

/// Material chip (assist / filter). `selected` toggles the filled tonal look.
#[component]
pub fn Chip(
    label: String,
    icon: Option<&'static str>,
    selected: Option<bool>,
    onclick: Option<EventHandler<MouseEvent>>,
    class: Option<&'static str>,
) -> Element {
    let selected = selected.unwrap_or(false);
    let class = class.unwrap_or("");
    rsx! {
        button {
            class: format!(
                "md-chip {} {class}",
                if selected { "md-chip-selected" } else { "" }
            ),
            onclick: move |e| {
                if let Some(handler) = onclick {
                    handler.call(e);
                }
            },
            if let Some(icon) = icon {
                span { class: "material-symbols-outlined md-chip-icon", "{icon}" }
            }
            span { "{label}" }
        }
    }
}

/// Small pill that reflects a status (`ok` / `warn` / `danger` / `muted`).
#[component]
pub fn StatusChip(status: &'static str, label: Option<&'static str>) -> Element {
    let label = label.unwrap_or(status);
    let tone = match status {
        "ok" | "connected" | "running" | "signed" | "graded" | "active" | "on-track"
        | "submitted" | "healthy" | "success" | "available" => "ok",
        "warn" | "pending" | "at-risk" | "flagged" | "review" | "degraded" => "warn",
        "danger" | "error" | "offline" | "failed" | "rejected" | "stopped" => "danger",
        _ => "muted",
    };
    rsx! {
        span { class: format!("md-status md-status-{tone}"),
            span { class: "md-status-dot" }
            "{label}"
        }
    }
}

/// Card with a labelled headline stat + trailing icon.
#[component]
pub fn StatCard(
    label: &'static str,
    value: String,
    icon: &'static str,
    tone: Option<&'static str>,
    delta: Option<&'static str>,
) -> Element {
    let tone = tone.unwrap_or("teal");
    rsx! {
        div { class: "md-card stat-card",
            div { class: "stat-card-top",
                span { class: format!("stat-icon stat-icon-{tone}"),
                    span { class: "material-symbols-outlined", "{icon}" }
                }
                if let Some(delta) = delta {
                    span { class: "stat-delta", "{delta}" }
                }
            }
            div { class: "stat-value", "{value}" }
            div { class: "stat-label", "{label}" }
        }
    }
}

/// Material linear progress indicator with tone classes.
#[component]
pub fn ProgressBar(value: f64, tone: Option<&'static str>) -> Element {
    let tone = tone.unwrap_or("teal");
    rsx! {
        div { class: format!("md-progress md-progress-{tone}"),
            div {
                class: "md-progress-fill",
                style: format!("width: {:.0}%", value.min(100.0).max(0.0)),
            }
        }
    }
}

/// Section heading with optional action element.
#[component]
pub fn SectionTitle(
    title: &'static str,
    subtitle: Option<&'static str>,
    action: Option<Element>,
) -> Element {
    rsx! {
        div { class: "md-section-head",
            div {
                h2 { "{title}" }
                if let Some(subtitle) = subtitle {
                    p { "{subtitle}" }
                }
            }
            if let Some(action) = action {
                div { class: "md-section-action", {action} }
            }
        }
    }
}

/// A simple Material card wrapper.
#[component]
pub fn Card(class: Option<&'static str>, children: Element) -> Element {
    let class = class.unwrap_or("");
    rsx! {
        div { class: format!("md-card {class}"), {children} }
    }
}

/// Divider line.
pub fn Divider(class: Option<&'static str>) -> Element {
    let class = class.unwrap_or("");
    rsx! { hr { class: format!("md-divider {class}") } }
}

/// Full-width empty/under-construction state.
#[component]
pub fn Placeholder(title: &'static str, desc: &'static str, icon: &'static str) -> Element {
    rsx! {
        div { class: "md-card placeholder",
            span { class: "material-symbols-outlined placeholder-icon", "{icon}" }
            h3 { "{title}" }
            p { "{desc}" }
            Button { variant: "tonal", icon: "construction", onclick: |_| {}, "Under construction" }
        }
    }
}
