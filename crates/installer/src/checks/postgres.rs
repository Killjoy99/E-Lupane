use crate::report::CheckResult;
use crate::wsl;

#[derive(Debug)]
pub struct PostgresStatus {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
    pub role_ready: bool,
    pub database_ready: bool,
}

pub fn check() -> CheckResult {
    let status = get_status();

    match (status.installed, status.running) {
        (true, true) => CheckResult::success(
            "PostgreSQL",
            format!(
                "installed & running ({})",
                status.version.unwrap_or_else(|| "unknown version".into())
            ),
        ),
        (true, false) => CheckResult::failure(
            "PostgreSQL",
            "installed but not running",
        ),
        _ => CheckResult::failure(
            "PostgreSQL",
            "not installed",
        ),
    }
}

/// Verify the application role + database have been provisioned.
pub fn check_config() -> CheckResult {
    let status = get_status();

    if !status.installed {
        return CheckResult::failure("PostgreSQL (elupane)", "PostgreSQL not installed");
    }

    match (status.role_ready, status.database_ready) {
        (true, true) => CheckResult::success(
            "PostgreSQL (elupane)",
            format!("role '{}' + database '{}' ready", crate::DB_USER, crate::DB_NAME),
        ),
        (false, _) => CheckResult::failure(
            "PostgreSQL (elupane)",
            format!("role '{}' not created", crate::DB_USER),
        ),
        _ => CheckResult::failure(
            "PostgreSQL (elupane)",
            format!("database '{}' not created", crate::DB_NAME),
        ),
    }
}

pub fn get_status() -> PostgresStatus {
    PostgresStatus {
        installed: is_installed(),
        running: is_running(),
        version: get_version(),
        role_ready: role_exists(),
        database_ready: database_exists(),
    }
}

pub fn is_installed() -> bool {
    wsl::run_ok("command -v psql >/dev/null 2>&1 && command -v pg_lsclusters >/dev/null 2>&1")
}

pub fn is_running() -> bool {
    wsl::run_ok("pg_isready -h 127.0.0.1 -p 5432")
}

pub fn get_version() -> Option<String> {
    let version = wsl::run_output("psql --version");
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

/// Check whether the `elupane` role exists (as the postgres superuser over the
/// local unix socket via peer auth).
pub fn role_exists() -> bool {
    wsl::run_root_output(&format!(
        "su -s /bin/bash postgres -c \"psql -Atc \\\"SELECT 1 FROM pg_roles WHERE rolname='{}'\\\"\"",
        crate::DB_USER
    )) == "1"
}

/// Check whether the `elupane` database exists.
pub fn database_exists() -> bool {
    wsl::run_root_output(&format!(
        "su -s /bin/bash postgres -c \"psql -Atc \\\"SELECT 1 FROM pg_database WHERE datname='{}'\\\"\"",
        crate::DB_NAME
    )) == "1"
}
