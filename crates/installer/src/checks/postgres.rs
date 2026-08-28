use std::process::Command;

use crate::report::CheckResult;

#[derive(Debug)]
pub struct PostgresStatus {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
}

pub fn check() -> CheckResult {
    let status = get_status();

    match (status.installed, status.running) {
        (true, true) => CheckResult::success(
            "PostgreSQL",
            format!(
                "Installed and running ({})",
                status.version.unwrap_or_else(|| "unknown".into())
            ),
        ),

        (true, false) => CheckResult::failure(
            "PostgreSQL",
            "Installed but not running",
        ),

        _ => CheckResult::failure(
            "PostgreSQL",
            "Not installed",
        ),
    }
}

pub fn get_status() -> PostgresStatus {
    let installed = is_installed();
    let running = is_running();
    let version = get_version();

    PostgresStatus {
        installed,
        running,
        version,
    }
}

pub fn is_installed() -> bool {
    Command::new("psql")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn is_running() -> bool {
    Command::new("pg_isready")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_version() -> Option<String> {
    let output = Command::new("psql")
        .arg("--version")
        .output()
        .ok()?;

    let version =
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

    Some(version)
}