use crate::report::CheckResult;
use crate::wsl;

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
                "installed & running ({})",
                status.version.unwrap_or_else(|| "unknown version".into())
            ),
        ),
        (true, false) => CheckResult::failure(
            "Redis",
            "installed but not running",
        ),
        _ => CheckResult::failure(
            "Redis",
            "not installed",
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
    wsl::run_ok("command -v redis-server >/dev/null 2>&1")
}

pub fn is_running() -> bool {
    wsl::run_output("redis-cli -h 127.0.0.1 ping").contains("PONG")
}

pub fn get_version() -> Option<String> {
    let version = wsl::run_output("redis-server --version");
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}