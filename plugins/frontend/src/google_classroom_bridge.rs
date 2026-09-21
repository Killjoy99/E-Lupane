//! Server-only Google OAuth bridge ("Sign in with Google").
//!
//! The browser never talks to Google directly. The fullstack server exposes the
//! OAuth 2.0 authorization-code flow as plain Axum routes and turns the Google
//! profile into a first-class platform session via the auth plugin:
//!
//! ```text
//! browser ──GET /api/auth/google/start──▶ 302 accounts.google.com (consent)
//! google ──GET /auth/google/callback?code─▶ server exchanges code for tokens,
//!                                           fetches userinfo, creates/updates
//!                                           user in auth plugin's users table,
//!                                           mints a JWT token and 302s back
//!                                           to /?token=…
//! ```
//!
//! Roles come from a local permission matrix of `.env` allow-lists (first
//! match wins): `GOOGLE_CLASSROOM_SUPER_ADMINS` → `ADMINS` → `DEVELOPERS` →
//! `LECTURERS`. Any other verified Google account is a `student`.
//!
//! Username/password login (via the auth plugin) is untouched — `/api/auth/verify`
//! accepts both token kinds so the two sign-in paths coexist.

use anyhow::{anyhow, Result};
use dioxus::server::axum::extract::Query;
use dioxus::server::axum::response::Redirect;
use serde::{Deserialize, Serialize};

const AUTHZ_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";

const SCOPE: &str = "openid email profile";

/// Single-use CSRF `state` values live for 10 minutes.
const STATE_TTL_SECS: u64 = 10 * 60;

const STATE_PREFIX: &str = "elupane:google:state:";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The Google profile (`openidconnect.googleapis.com/v1/userinfo`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUser {
    /// Google's stable account id (`sub`).
    pub sub: String,
    pub email: String,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub picture: Option<String>,
}

/// The local projection returned by the auth plugin for Google users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub active: bool,
    pub roles: Vec<String>,
}

// ---------------------------------------------------------------------------
// Local permission matrix
// ---------------------------------------------------------------------------


/// Exchange authorization code for tokens.
pub async fn exchange_code(code: &str) -> anyhow::Result<GoogleTokens> {
    let client_id = std::env::var("GOOGLE_OAUTH_CLIENT_ID")?;
    let client_secret = std::env::var("GOOGLE_OAUTH_CLIENT_SECRET")?;
    let redirect_uri = std::env::var("GOOGLE_OAUTH_REDIRECT_URI")?;

    let client = reqwest::Client::new();
    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("redirect_uri", &redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await?;
        return Err(anyhow!("token exchange failed: {}", body));
    }

    let tokens: GoogleTokens = resp.json().await?;
    Ok(tokens)
}

/// Fetch user info from Google.
pub async fn fetch_user_info(access_token: &str) -> anyhow::Result<GoogleUser> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await?;
        return Err(anyhow!("userinfo fetch failed: {}", body));
    }

    let user: GoogleUser = resp.json().await?;
    Ok(user)
}

/// Google OAuth tokens response.
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleTokens {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

/// Generate Google OAuth authorization URL.
pub fn build_auth_url(state: &str) -> String {
    let client_id = std::env::var("GOOGLE_OAUTH_CLIENT_ID").unwrap_or_default();
    let redirect_uri = std::env::var("GOOGLE_OAUTH_REDIRECT_URI").unwrap_or_default();

    format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&state={}&access_type=offline&prompt=consent",
        urlencode(&client_id),
        urlencode(&redirect_uri),
        urlencode(state)
    )
}

/// Read a comma-separated `.env` allow-list (trimmed, lower-cased).
fn allow_list(var: &str) -> Vec<String> {
    std::env::var(var)
        .unwrap_or_default()
        .split(',')


        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Resolve the platform role(s) for a Google email. First matching tier wins;
/// a tier grants every role in its set so holders can switch dashboards
/// (e.g. `super_admin` → superadmin + admin + dean + hod + lecturer). Everyone
/// else is a `student`. Case-insensitive.
pub fn resolve_role(email: &str) -> Vec<String> {
    let email = email.trim().to_ascii_lowercase();
    const MATRIX: [(&str, &[&str]); 6] = [
        (
            "GOOGLE_CLASSROOM_SUPER_ADMINS",
            &["super_admin", "admin", "dean", "hod", "lecturer"],
        ),
        ("GOOGLE_CLASSROOM_ADMINS", &["admin", "dean", "hod", "lecturer"]),
        ("GOOGLE_CLASSROOM_DEANS", &["dean", "hod", "lecturer"]),
        ("GOOGLE_CLASSROOM_DEVELOPERS", &["developer"]),
        ("GOOGLE_CLASSROOM_HODS", &["hod", "lecturer"]),
        ("GOOGLE_CLASSROOM_LECTURERS", &["lecturer"]),
    ];
    for (var, roles) in MATRIX {
        if allow_list(var).contains(&email) {
            return roles.iter().map(|r| r.to_string()).collect();
        }
    }
    vec!["student".to_string()]
}

/// Dev bypass flag: `GOOGLE_DEV_LOGIN=1` in the environment enables
/// `GET /api/auth/google/dev_login?email=…` for smoke tests.
pub fn dev_login_enabled() -> bool {
    std::env::var("GOOGLE_DEV_LOGIN")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn env_req(var: &str) -> Result<String> {
    std::env::var(var).map_err(|_| anyhow!("{var} is not set (check .env)"))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// `given_name + family_name`, falling back to `name`, then the email itself.
fn display_name(user: &GoogleUser) -> String {
    let given = user.given_name.as_deref().unwrap_or_default().trim();
    let family = user.family_name.as_deref().unwrap_or_default().trim();
    let combined = format!("{given} {family}").trim().to_string();
    if !combined.is_empty() {
        return combined;
    }
    let name = user.name.as_deref().unwrap_or_default().trim();
    if !name.is_empty() {
        return name.to_string();
    }
    user.email.clone()
}

// ---------------------------------------------------------------------------
// Public API used by server functions / routes
// ---------------------------------------------------------------------------

/// Sign in (or refresh) a Google-backed account: delegate to the auth plugin,
/// which creates the user in the central `users` table if needed and returns a
/// JWT token.
pub async fn login_or_create(user: &GoogleUser) -> Result<(String, LocalUser)> {
    if !user.email_verified {
        return Err(anyhow!("Google account email is not verified"));
    }
    let roles = resolve_role(&user.email);
    let full_name = display_name(user);
    let (token, view) = crate::auth_bridge::google_login(&user.email, &full_name, roles.clone()).await?;
    let local = LocalUser {
        id: view.id,
        username: view.username,
        email: view.email,
        full_name: view.full_name,
        active: view.active,
        roles: view.roles,
    };
    Ok((token, local))
}

/// Validate a Google session token. For new sessions this is a JWT verified
/// through the auth plugin; legacy opaque Redis tokens are no longer valid
/// and will return an error (the caller falls back to re-login).
pub async fn verify_session(token: &str) -> Result<LocalUser> {
    let view = crate::auth_bridge::verify(token).await?;
    let user = view.ok_or_else(|| anyhow!("session expired or invalid"))?;
    Ok(LocalUser {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        active: user.active,
        roles: user.roles,
    })
}

/// Revoke a Google session. JWT-backed sessions cannot be individually revoked
/// (they expire client-side), so this is a no-op.
pub async fn logout(_token: &str) -> Result<()> {
    Ok(())
}

// ---------------------------------------------------------------------------
// Admin JWT minting (bridge to the auth plugin's RBAC admin API)
// ---------------------------------------------------------------------------

/// Mint an auth-plugin-compatible admin JWT (HS256, shared `AUTH_JWT_SECRET`)
/// for a Google-backed user holding an admin-tier role. The auth plugin's
/// `require_admin` validates these tokens, so Google admins can manage
/// password users, roles and the permission matrix without a password login.
pub fn mint_admin_jwt(user: &LocalUser) -> Option<String> {
    let is_admin = user
        .roles
        .iter()
        .any(|r| r == "admin" || r == "super_admin");
    if !is_admin {
        return None;
    }

    #[derive(serde::Serialize)]
    struct AdminClaims {
        sub: String,
        username: String,
        roles: Vec<String>,
        exp: usize,
    }

    let exp = (chrono::Utc::now() + chrono::Duration::hours(12)).timestamp() as usize;
    let secret =
        std::env::var("AUTH_JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &AdminClaims {
            sub: user.id.clone(),
            username: user.username.clone(),
            roles: user.roles.clone(),
            exp,
        },
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .ok()
}

// ---------------------------------------------------------------------------
// OAuth flow helpers
// ---------------------------------------------------------------------------

/// Build the Google authorization URL for a fresh (Redis-stored) CSRF state.
async fn begin_login() -> Result<String> {
    let state = uuid::Uuid::new_v4().to_string();
    cache::session::set(&format!("{STATE_PREFIX}{state}"), "1", STATE_TTL_SECS).await?;
    build_authorize_url(&state)
}

fn build_authorize_url(state: &str) -> Result<String> {
    let client_id = env_req("GOOGLE_OAUTH_CLIENT_ID")?;
    let redirect_uri = env_req("GOOGLE_OAUTH_REDIRECT_URI")?;
    Ok(format!(
        "{AUTHZ_URL}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&prompt=select_account",
        urlencode(&client_id),
        urlencode(&redirect_uri),
        urlencode(SCOPE),
        urlencode(state),
    ))
}

/// Exchange the authorization code for tokens, then fetch the userinfo profile.
async fn exchange_code_and_profile(code: &str) -> Result<GoogleUser> {
    let client_id = env_req("GOOGLE_OAUTH_CLIENT_ID")?;
    let client_secret = env_req("GOOGLE_OAUTH_CLIENT_SECRET")?;
    let redirect_uri = env_req("GOOGLE_OAUTH_REDIRECT_URI")?;

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
    }

    let token: TokenResponse = reqwest::Client::new()
        .post(TOKEN_URL)
        .form(&[
            ("code", code),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let user: GoogleUser = reqwest::Client::new()
        .get(USERINFO_URL)
        .bearer_auth(&token.access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(user)
}

/// Minimal percent-encoding for URL query values.
fn urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn redirect_with_error(message: &str) -> Redirect {
    Redirect::to(&format!("/?auth_error={}", urlencode(message)))
}

// ---------------------------------------------------------------------------
// Axum routes (registered in `main.rs`)
// ---------------------------------------------------------------------------

/// `GET /api/auth/google/start` — kick off the consent flow.
pub async fn start_handler() -> Redirect {
    match begin_login().await {
        Ok(url) => Redirect::to(&url),
        Err(err) => redirect_with_error(&err.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

/// `GET /auth/google/callback` — finish the consent flow (must match
/// `GOOGLE_OAUTH_REDIRECT_URI`).
pub async fn callback_handler(Query(q): Query<CallbackQuery>) -> Redirect {
    if let Some(err) = q.error.as_deref() {
        return redirect_with_error(&format!("Google sign-in was cancelled: {err}"));
    }
    let (Some(code), Some(state)) = (q.code, q.state) else {
        return redirect_with_error("missing authorization code or state");
    };

    // Validate + consume the single-use CSRF state.
    let state_key = format!("{STATE_PREFIX}{state}");
    let valid = cache::session::get(&state_key).await.ok().flatten().is_some();
    let _ = cache::session::delete(&state_key).await;
    if !valid {
        return redirect_with_error("login link expired — please try again");
    }

    match exchange_code_and_profile(&code).await {
        Ok(user) => match login_or_create(&user).await {
            Ok((token, _)) => Redirect::to(&format!("/?token={}", urlencode(&token))),
            Err(err) => redirect_with_error(&err.to_string()),
        },
        Err(err) => redirect_with_error(&err.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct DevLoginQuery {
    email: String,
    #[serde(default)]
    name: Option<String>,
}

/// `GET /api/auth/google/dev_login?email=…` — bypass Google entirely (only
/// when `GOOGLE_DEV_LOGIN=1`). Synthesizes the same profile shape so the rest
/// of the flow delegates to the auth plugin.
pub async fn dev_login_handler(Query(q): Query<DevLoginQuery>) -> Redirect {
    if !dev_login_enabled() {
        return redirect_with_error("dev login is disabled");
    }
    let email = q.email.trim().to_string();
    if email.is_empty() || !email.contains('@') {
        return redirect_with_error("dev login needs a valid ?email=…");
    }
    eprintln!("[auth] DEV login for {email}");
    let full = q.name.clone().unwrap_or_default();
    let (given, family) = match full.split_once(' ') {
        Some((g, f)) => (g.to_string(), f.to_string()),
        None => (full.clone(), String::new()),
    };
    let user = GoogleUser {
        sub: format!("dev:{email}"),
        email,
        email_verified: true,
        name: if full.is_empty() { None } else { Some(full) },
        given_name: if given.is_empty() { None } else { Some(given) },
        family_name: if family.is_empty() { None } else { Some(family) },
        picture: None,
    };
    match login_or_create(&user).await {
        Ok((token, _)) => Redirect::to(&format!("/?token={}", urlencode(&token))),
        Err(err) => redirect_with_error(&err.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Tests (pure logic — no Redis / Postgres / network)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn set_matrix(super_admins: &str, admins: &str, developers: &str, lecturers: &str) {
        std::env::set_var("GOOGLE_CLASSROOM_SUPER_ADMINS", super_admins);
        std::env::set_var("GOOGLE_CLASSROOM_ADMINS", admins);
        std::env::set_var("GOOGLE_CLASSROOM_DEVELOPERS", developers);
        std::env::set_var("GOOGLE_CLASSROOM_LECTURERS", lecturers);
    }

    #[test]
    fn role_matrix_grants_full_stack_to_super_admin() {
        let _guard = ENV_LOCK.lock().unwrap();
        set_matrix("root@x.com", "root@x.com", "", "");
        assert_eq!(
            resolve_role("root@x.com"),
            vec!["super_admin", "admin", "lecturer"]
        );
    }

    #[test]
    fn role_matrix_defaults_to_student() {
        let _guard = ENV_LOCK.lock().unwrap();
        set_matrix("", "", "", "");
        assert_eq!(resolve_role("anyone@x.com"), vec!["student"]);
    }

    #[test]
    fn resolve_role_is_case_insensitive() {
        let _guard = ENV_LOCK.lock().unwrap();
        set_matrix("", "Mixed.Case@X.com", "", "");
        assert_eq!(
            resolve_role("mixed.case@x.com"),
            vec!["admin", "lecturer"]
        );
    }

    #[test]
    fn display_name_prefers_given_and_family() {
        let user = GoogleUser {
            sub: "s".into(),
            email: "a@b.com".into(),
            email_verified: true,
            name: Some("A B".into()),
            given_name: Some("A".into()),
            family_name: Some("B".into()),
            picture: None,
        };
        assert_eq!(display_name(&user), "A B");
    }

    #[test]
    fn urlencode_escapes_specials() {
        assert_eq!(urlencode("a b&c=d"), "a%20b%26c%3Dd");
    }
}





