//! Generate / verify the root `.env` with the database and Redis connection
//! strings that the rest of the system reads.

use std::path::Path;

/// Ensure the root `.env` contains `DATABASE_URL` and `REDIS_URL` pointing at
/// the provisioned services. Idempotent: existing keys are replaced in place,
/// comments and unrelated keys are preserved.
pub fn ensure_env() -> anyhow::Result<()> {
    let path = Path::new(".env");

    let user = std::env::var("DB_USER").unwrap_or_else(|_| crate::DB_USER.to_string());
    let password = std::env::var("DB_PASSWORD").unwrap_or_else(|_| crate::DB_PASSWORD.to_string());
    let db = std::env::var("DB_NAME").unwrap_or_else(|_| crate::DB_NAME.to_string());
    let port = std::env::var("DB_PORT").unwrap_or_else(|_| crate::DB_PORT.to_string());

    let database_url = format!("DATABASE_URL=postgres://{user}:{password}@localhost:{port}/{db}");
    let redis_url = "REDIS_URL=redis://127.0.0.1:6379".to_string();

    let mut lines: Vec<String> = if path.exists() {
        std::fs::read_to_string(path)?
            .lines()
            .map(str::to_string)
            .collect()
    } else {
        Vec::new()
    };

    set_or_add(&mut lines, "DATABASE_URL", &database_url);
    set_or_add(&mut lines, "REDIS_URL", &redis_url);

    let mut out = lines.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }

    std::fs::write(path, out)?;
    tracing::info!("Wrote .env: {database_url}");
    Ok(())
}

fn set_or_add(lines: &mut Vec<String>, key: &str, value: &str) {
    let prefix = format!("{key}=");
    match lines.iter().position(|line| line.starts_with(&prefix)) {
        Some(pos) => lines[pos] = value.to_string(),
        None => lines.push(value.to_string()),
    }
}

