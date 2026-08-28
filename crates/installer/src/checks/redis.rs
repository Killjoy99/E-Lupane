use std::process::Command;

use crate::report::CheckResult;

#[derive(Debug)]
pub struct RedisStatus {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
}

pub fn check() -> CheckResult {
    let status = get_status();

    match (status.installed, status.running) {
        (true, true) => CheckResult::success(
            "Redis",
            format!(
                "Installed and running ({})",
                status.version.unwrap_or_else(|| "unknown".into())
            ),
        ),

        (true, false) => CheckResult::failure(
            "Redis",
            "Installed but not running",
        ),

        _ => CheckResult::failure(
            "Redis",
            "Not installed",
        ),
    }
}

pub fn get_status() -> RedisStatus {
    RedisStatus {
        installed: is_installed(),
        running: is_running(),
        version: get_version(),
    }
}

pub fn is_installed() -> bool {
    Command::new("redis-server")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn is_running() -> bool {
    let output = Command::new("redis-cli")
        .arg("ping")
        .output();

    match output {
        Ok(output) => {
            let stdout =
                String::from_utf8_lossy(&output.stdout);

            stdout.contains("PONG")
        }

        Err(_) => false,
    }
}

pub fn get_version() -> Option<String> {
    let output = Command::new("redis-server")
        .arg("--version")
        .output()
        .ok()?;

    let version =
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

    Some(version)
}