//! Live RBAC administration pages (replacing the static mock admin screens).
//!
//! * [`AdminUsersPage`] — search every account by name/username/email, grant or
//!   revoke roles per user, create and delete password accounts.
//! * [`AdminPermissionsPage`] — the roles × permissions matrix, editable in
//!   place: toggling a cell rewrites that role's permission set via the auth
//!   plugin's `role.permissions.set` RPC.
//!
//! Both pages require an admin credential. For username/password admins that is
//! their login JWT; for Google admins it is the admin JWT minted at sign-in
//! (`AuthSession::admin_token`). Non-admins see a gate instead of the tables.

use dioxus::prelude::*;

use crate::dashboard::UiCtx;
use crate::server_functions::{
        admin_permissions_list, admin_role_grant, admin_role_permissions_set,
    admin_role_revoke, admin_roles_list, users_create, users_delete, users_list, AuthUser,
    CreateUserInput, PermissionDefView, RolePermissionsView,
};
use crate::components::{Button, SectionTitle, StatCard};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Icon for a role name (mirrors the mock page's mapping).
pub(crate) fn role_icon(role: &str) -> &'static str {
    match role {
        "super_admin" => "shield_person",
        "admin" => "admin_panel_settings",
        "lecturer" => "co_present",
        "developer" => "code",
        "student" => "school",
        _ => "person",
    }
}

/// Friendly display label for a role name (public for the users table chips).
pub(crate) fn role_display(role: &str) -> String {
    role_label(role)
}

/// Friendly display label for a role name.
fn role_label(role: &str) -> String {
    role.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The admin credential for the current session (`None` for non-admins).
fn admin_token_of(ctx: &UiCtx) -> Option<String> {
    (ctx.session)().and_then(|s| s.admin_token)
}

/// Gate rendered when the signed-in user has no admin credential.
fn AdminGate() -> Element {
    rsx! {
        div { class: "page",
            SectionTitle { title: "Administration", subtitle: "RBAC management" }
            div { class: "md-card admin-gate",
                span { class: "material-symbols-outlined", "lock" }
                p { "You need an admin role to manage users and permissions." }
                p { class: "muted", "Ask a super admin to grant your account the admin role, or sign in with an admin account." }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Users management (live)
// ---------------------------------------------------------------------------

/// Live user management: search, role grant/revoke, create + delete accounts.
#[component]
pub fn AdminUsersPage() -> Element {
    let ctx = use_context::<UiCtx>();
    let admin_token = admin_token_of(&ctx);

    if admin_token.is_none() {
        return rsx! { AdminGate {} };
    }
    let admin_token = admin_token.unwrap();

    let mut users: Signal<Vec<AuthUser>> = use_signal(Vec::new);
    let mut roles: Signal<Vec<RolePermissionsView>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut notice: Signal<Option<String>> = use_signal(|| None);
    let mut search = use_signal(String::new);
    let mut refresh = use_signal(|| 0u32);
    // Per-row in-flight flag (indexed by user id) + row action errors.
    let mut row_busy: Signal<Option<String>> = use_signal(|| None);

    // Initial load + reloads after mutations.
    {
        let admin_token = admin_token.clone();
        use_future(move || {
            let token = admin_token.clone();
            async move {
                refresh(); // re-run when bumped
                loading.set(true);
                error.set(None);
                match users_list(token.clone()).await {
                    Ok(list) => users.set(list),
                    Err(e) => error.set(Some(e.to_string())),
                }
                match admin_roles_list(Some(token)).await {
                    Ok(list) => roles.set(list),
                    Err(e) => error.set(Some(e.to_string())),
                }
                loading.set(false);
            }
        });
    }

    // "Add user" form visibility (the form owns its own field state).
    let mut show_create = use_signal(|| false);

    let q = search().trim().to_lowercase();
    let all_roles: Vec<String> = roles().iter().map(|r| r.name.clone()).collect();

    rsx! {
        div { class: "page",
            SectionTitle { title: "Users", subtitle: "Every account on the platform — searchable by name, username or email" }

            if let Some(err) = error() {
                div { class: "auth-error admin-banner",
                    span { class: "material-symbols-outlined", "error" }
                    span { "{err}" }
                }
            }
            if let Some(msg) = notice() {
                div { class: "admin-banner ok",
                    span { class: "material-symbols-outlined", "check_circle" }
                    span { "{msg}" }
                }
            }

            div { class: "grid-4",
                StatCard { label: "Total users", value: format!("{}", users().len()), icon: "group", tone: "teal", delta: None }
                StatCard { label: "Active", value: format!("{}", users().iter().filter(|u| u.active).count()), icon: "check_circle", tone: "green", delta: None }
                StatCard { label: "Suspended", value: format!("{}", users().iter().filter(|u| !u.active).count()), icon: "block", tone: "red", delta: None }
                StatCard { label: "Roles", value: format!("{}", roles().len()), icon: "badge", tone: "indigo", delta: None }
            }

            div { class: "md-card table-card",
                div { class: "table-toolbar",
                    div { class: "admin-search",
                        span { class: "material-symbols-outlined", "search" }
                        input {
                            r#type: "search",
                            placeholder: "Search name, username or email…",
                            value: "{search}",
                            oninput: move |e| search.set(e.value()),
                        }
                    }
                    Button { variant: "filled", icon: "person_add", onclick: move |_| show_create.toggle(), "Add user" }
                }

                if show_create() {
                    AdminCreateForm {
                        admin_token: admin_token.clone(),
                        roles: all_roles.clone(),
                        on_done: move |msg: String| {
                            notice.set(Some(msg));
                            error.set(None);
                            show_create.set(false);
                            refresh += 1;
                        },
                        on_error: move |msg: String| {
                            error.set(Some(msg));
                            notice.set(None);
                        },
                    }
                }

                if loading() {
                    div { class: "admin-loading", span { class: "auth-spinner" } }
                } else {
                    AdminUsersTable {
                        admin_token,
                        users,
                        all_roles,
                        row_busy,
                        on_error: move |msg: String| { error.set(Some(msg)); notice.set(None); },
                        on_notice: move |msg: String| { notice.set(Some(msg)); error.set(None); },
                    }
                }

            }
        }
    }
}

/// Swap an updated user into the cached list (client-side refresh).
fn replace_user(users: &mut Signal<Vec<AuthUser>>, id: &str, updated: AuthUser) {
    users.with_mut(|v| {
        if let Some(slot) = v.iter_mut().find(|x| x.id == id) {
            *slot = updated;
        }
    });
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
        out.push('?');
    }
    out
}

// ---------------------------------------------------------------------------
// Create-user form
// ---------------------------------------------------------------------------

/// Inline "Add user" form: username/email are unique 1:1 (enforced by the auth
/// plugin's schema); roles are picked as toggle chips.
#[component]
fn AdminCreateForm(
    admin_token: String,
    roles: Vec<String>,
    on_done: EventHandler<String>,
    on_error: EventHandler<String>,
) -> Element {
    let mut username = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut full_name = use_signal(String::new);
    let mut picked: Signal<Vec<String>> = use_signal(|| vec!["student".to_string()]);
    let mut busy = use_signal(|| false);

    rsx! {
        div { class: "admin-create-form",
            div { class: "admin-create-grid",
                div { class: "auth-field",
                    label { "Username *" }
                    input {
                        r#type: "text",
                        placeholder: "unique username",
                        value: "{username}",
                        oninput: move |e| username.set(e.value()),
                    }
                }
                div { class: "auth-field",
                    label { "Email *" }
                    input {
                        r#type: "email",
                        placeholder: "unique email",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                }
                div { class: "auth-field",
                    label { "Full name" }
                    input {
                        r#type: "text",
                        placeholder: "Thandiwe Ndlovu",
                        value: "{full_name}",
                        oninput: move |e| full_name.set(e.value()),
                    }
                }
                div { class: "auth-field",
                    label { "Password *" }
                    input {
                        r#type: "password",
                        placeholder: "••••••••",
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                    }
                }
            }

            div { class: "admin-role-pick",
                for r in roles.iter() {
                    {
                        let r = r.clone();
                        let on = picked().contains(&r);
                        rsx! {
                            button {
                                class: if on { "role-chip on" } else { "role-chip" },
                                r#type: "button",
                                onclick: move |_| {
                                    let mut next = picked();
                                    if next.contains(&r) {
                                        next.retain(|x| x != &r);
                                    } else {
                                        next.push(r.clone());
                                    }
                                    picked.set(next);
                                },
                                span { class: "material-symbols-outlined", "{role_icon(&r)}" }
                                span { "{role_label(&r)}" }
                            }
                        }
                    }
                }
            }

            div { class: "admin-create-actions",
                Button {
                    variant: "filled",
                    icon: "person_add",
                    disabled: busy(),
                    onclick: move |_| {
                        let token = admin_token.clone();
                        async move {
                            let uname = username().trim().to_string();
                            let mail = email().trim().to_string();
                            if uname.is_empty() || mail.is_empty() || password().is_empty() {
                                on_error.call("Username, email and password are required.".into());
                                return;
                            }
                            busy.set(true);
                            let input = CreateUserInput {
                                username: uname,
                                email: mail,
                                password: password(),
                                full_name: full_name().trim().to_string(),
                                roles: picked(),
                            };
                            match users_create(token, input).await {
                                Ok(u) => on_done.call(format!("Created account “{}”.", u.username)),
                                Err(e) => on_error.call(e.to_string()),
                            }
                            busy.set(false);
                        }
                    },
                    if busy() { "Creating…" } else { "Create user" }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Users table (search results + per-row role management)
// ---------------------------------------------------------------------------

/// One row per account: identity, status, role chips (click to grant/revoke)
/// and a delete action. Roles the user already holds show as `on` chips.
#[component]
fn AdminUsersTable(
    admin_token: String,
    users: Signal<Vec<AuthUser>>,
    all_roles: Vec<String>,
    row_busy: Signal<Option<String>>,
    on_error: EventHandler<String>,
    on_notice: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "admin-table-wrap",
            table { class: "admin-table",
                thead {
                    tr {
                        th { "User" }
                        th { "Username" }
                        th { "Email" }
                        th { "Roles" }
                        th { "Status" }
                        th { "" }
                    }
                }
                tbody {
                    for user in users().iter().cloned() {
                        AdminUserRow {
                            admin_token: admin_token.clone(),
                            user: user.clone(),
                            all_roles: all_roles.clone(),
                            row_busy,
                            on_error,
                            on_notice,
                            users,
                        }
                    }
                    if users().is_empty() {
                        tr {
                            td { class: "admin-empty", colspan: "6", "No accounts match the search." }
                        }
                    }
                }
            }
        }
    }
}

/// One table row: identity, status, role chips (click to grant/revoke) and a
/// delete action. Chips the user already holds are `on`; clicking any chip
/// toggles that role via the auth plugin's `role.grant` / `role.revoke`.
#[component]
fn AdminUserRow(
    admin_token: String,
    user: AuthUser,
    all_roles: Vec<String>,
    row_busy: Signal<Option<String>>,
    on_error: EventHandler<String>,
    on_notice: EventHandler<String>,
    users: Signal<Vec<AuthUser>>,
) -> Element {
    let user_id = user.id.clone();
    let busy_here = row_busy().as_deref() == Some(user_id.as_str());

    rsx! {
        tr { class: if busy_here { "admin-row busy" } else { "admin-row" },
            td {
                div { class: "admin-user-cell",
                    span { class: "admin-user-name", "{user.full_name}" }
                }
            }
            td { class: "admin-mono", "{user.username}" }
            td { class: "admin-mono", "{user.email}" }
            td {
                div { class: "admin-chip-row",
                    for role in all_roles.iter() {
                        {
                            let admin_token = admin_token.clone();
                            let user_id = user_id.clone();
                            let held = user.roles.iter().any(|r| r == role);
                            let role = role.clone();
                            let held_label = if held { "revoke" } else { "grant" };
                            rsx! {
                                button {
                                    class: if held { "admin-chip on" } else { "admin-chip" },
                                    title: format!("{held_label} {role}"),
                                    disabled: busy_here,
                                    onclick: move |_| {
                                        let token = admin_token.clone();
                                        let uid = user_id.clone();
                                        let role = role.clone();
                                        let held = held;
                                        async move {
                                            row_busy.set(Some(uid.clone()));
                                            let result = if held {
                                                admin_role_revoke(Some(token), uid.clone(), role.clone()).await
                                            } else {
                                                admin_role_grant(Some(token), uid.clone(), role.clone()).await
                                            };
                                            match result {
                                                Ok(updated) => {
                                                    // Swap the refreshed user into the list.
                                                    let mut list = users();
                                                    if let Some(slot) = list.iter_mut().find(|u| u.id == uid) {
                                                        *slot = updated;
                                                    }
                                                    users.set(list);
                                                    on_notice.call(if held {
                                                        format!("Revoked “{role}”.")
                                                    } else {
                                                        format!("Granted “{role}”.")
                                                    });
                                                }
                                                Err(e) => on_error.call(e.to_string()),
                                            }
                                            row_busy.set(None);
                                        }
                                    },
                                    span { class: "material-symbols-outlined", { role_icon(&role) } }
                                    span { "{role_display(&role)}" }
                                }
                            }
                        }
                    }
                    if user.roles.iter().any(|r| !all_roles.contains(r)) {
                        span { class: "admin-chip orphan", title: "Role not in the role list", "…" }
                    }
                }
            }
            td {
                span {
                    class: if user.active { "admin-status ok" } else { "admin-status off" },
                    if user.active { "active" } else { "disabled" }
                }
            }
            td {
                {
                    let admin_token = admin_token.clone();
                    let user_id = user_id.clone();
                    let uname = user.username.clone();
                    rsx! {
                        button {
                            class: "md-icon-btn admin-danger",
                            title: "Delete account",
                            disabled: busy_here,
                            onclick: move |_| {
                                let token = admin_token.clone();
                                let uid = user_id.clone();
                                let uname = uname.clone();
                                async move {
                                    row_busy.set(Some(uid.clone()));
                                    match users_delete(token, uid.clone()).await {
                                        Ok(()) => {
                                            users.set(users().into_iter().filter(|u| u.id != uid).collect());
                                            on_notice.call(format!("Deleted account “{uname}”."));
                                        }
                                        Err(e) => on_error.call(e.to_string()),
                                    }
                                    row_busy.set(None);
                                }
                            },
                            span { class: "material-symbols-outlined", "delete" }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Roles × permissions matrix (live)
// ---------------------------------------------------------------------------

/// The editable permission matrix: one row per role, one column per permission
/// definition. Toggling a cell immediately writes the role's full permission
/// set via the auth plugin's `role.permissions.set` RPC.
#[component]
pub fn AdminPermissionsPage() -> Element {
    let ctx = use_context::<UiCtx>();
    let admin_token = admin_token_of(&ctx);

    if admin_token.is_none() {
        return rsx! { AdminGate {} };
    }
    let admin_token = admin_token.unwrap();

    let mut roles: Signal<Vec<RolePermissionsView>> = use_signal(Vec::new);
    let mut perms: Signal<Vec<PermissionDefView>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut notice: Signal<Option<String>> = use_signal(|| None);
    let mut cell_busy: Signal<Option<(String, String)>> = use_signal(|| None);

    // Initial load: roles (with permission sets) + permission definitions.
    {
        let admin_token = admin_token.clone();
        use_future(move || {
            let token = admin_token.clone();
            async move {
                match (
                    admin_roles_list(Some(token.clone())).await,
                    admin_permissions_list(Some(token)).await,
                ) {
                    (Ok(r), Ok(p)) => {
                        roles.set(r);
                        perms.set(p);
                    }
                    (Err(e), _) | (_, Err(e)) => error.set(Some(e.to_string())),
                }
                loading.set(false);
            }
        });
    }

    rsx! {
        div { class: "page",
            SectionTitle {
                title: "Role permissions",
                subtitle: "Toggle what each role can do — saved instantly",
            }

            if loading() {
                div { class: "md-card admin-loading",
                    span { class: "auth-spinner" }
                    "Loading permission matrix…"
                }
            } else {
                if let Some(err) = error() {
                    div { class: "auth-error",
                        span { class: "material-symbols-outlined", "error" }
                        span { "{err}" }
                    }
                }
                if let Some(msg) = notice() {
                    div { class: "admin-notice",
                        span { class: "material-symbols-outlined", "check_circle" }
                        span { "{msg}" }
                    }
                }

                div { class: "admin-table-wrap admin-matrix-wrap",
                    table { class: "admin-table admin-matrix",
                        thead {
                            tr {
                                th { class: "admin-matrix-role", "Role" }
                                for p in perms().iter() {
                                    th {
                                        class: "admin-matrix-perm",
                                        title: p.description.clone().unwrap_or_default(),
                                        span { class: "admin-matrix-perm-name", "{role_display(&p.name)}" }
                                    }
                                }
                            }
                        }
                        tbody {
                            for role in roles().iter().cloned() {
                                AdminMatrixRow {
                                    admin_token: admin_token.clone(),
                                    role,
                                    roles,
                                    permissions: perms(),
                                    cell_busy,
                                    on_error: move |e| error.set(Some(e)),
                                    on_notice: move |m| notice.set(Some(m)),
                                }
                            }
                        }
                    }
                }
                p { class: "muted admin-matrix-hint",
                    "Rows are roles; columns are permissions. A ticked cell means the role holds that permission."
                }
            }
        }
    }
}
/// One matrix row: a role and its on/off cells for every permission. Toggling
/// a cell recomputes that role's full permission set (with the cell flipped)
/// and writes it via `role.permissions.set`.
#[component]
fn AdminMatrixRow(
    admin_token: String,
    role: RolePermissionsView,
    mut roles: Signal<Vec<RolePermissionsView>>,
    permissions: Vec<PermissionDefView>,
    cell_busy: Signal<Option<(String, String)>>,
    on_error: EventHandler<String>,
    on_notice: EventHandler<String>,
) -> Element {
    let role_name = role.name.clone();

    rsx! {
        tr {
            td { class: "admin-matrix-role",
                div { class: "admin-matrix-role-cell",
                    span { class: "material-symbols-outlined", { role_icon(&role_name) } }
                    span { "{role_display(&role_name)}" }
                }
            }
            for perm in permissions.iter() {
                {
                    let admin_token = admin_token.clone();
                    let role_name = role_name.clone();
                    let perm_id = perm.id.clone();
                    let held = role.permissions.iter().any(|p| *p == perm_id);
                    let busy_here =
                        cell_busy().as_ref() == Some(&(role_name.clone(), perm_id.clone()));
                    let perm_name = perm.name.clone();
                    // Snapshot this role's permission set for the toggle handler
                    // (avoids moving the component prop into the closure).
                    let role_perms: Vec<String> = role.permissions.clone();
                    rsx! {
                        td { class: "admin-matrix-cell",
                            button {
                                class: if held { "admin-cell on" } else { "admin-cell" },
                                title: format!(
                                    "{} “{}” {}",
                                    if held { "Remove" } else { "Grant" },
                                    perm_name,
                                    if held { "from" } else { "to" }),
                                disabled: busy_here,
                                onclick: move |_| {
                                    let token = admin_token.clone();
                                    let rn = role_name.clone();
                                    let pid = perm_id.clone();
                                    let pname = perm_name.clone();
                                    let held = held;
                                    let role_perms = role_perms.clone();
                                    async move {
                                        cell_busy.set(Some((rn.clone(), pid.clone())));
                                        // Full set with this cell flipped.
                                        let mut next: Vec<String> = role_perms
                                            .iter()
                                            .filter(|p| *p != &pid)
                                            .cloned()
                                            .collect();
                                        if !held {
                                            next.push(pid.clone());
                                        }
                                        match admin_role_permissions_set(
                                            Some(token),
                                            rn.clone(),
                                            next,
                                        )
                                        .await
                                        {
                                            Ok(updated) => {
                                                // Replace the row from the response.
                                                let mut list = roles();
                                                if let Some(slot) =
                                                    list.iter_mut().find(|r| r.name == rn)
                                                {
                                                    *slot = updated;
                                                }
                                                roles.set(list);
                                                on_notice.call(if held {
                                                    format!("Removed “{pname}” from {}.", role_display(&rn))
                                                } else {
                                                    format!("Granted “{pname}” to {}.", role_display(&rn))
                                                });
                                            }
                                            Err(e) => on_error.call(e.to_string()),
                                        }
                                        cell_busy.set(None);
                                    }
                                },
                                span { class: "material-symbols-outlined",
                                    if held { "check_box" } else { "check_box_outline_blank" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}




