use crate::package_manager::{effective_manager, PackageManager};
use crate::wsl;

/// Install PostgreSQL in the E-Lupane runtime environment (WSL on Windows,
/// the local distro on Linux).
pub fn install(manager: PackageManager) -> anyhow::Result<()> {
    match effective_manager(manager) {
        PackageManager::Apt => {
            wsl::run_root_check("apt-get update")?;
            wsl::run_root_check(
                "DEBIAN_FRONTEND=noninteractive apt-get install -y postgresql postgresql-client",
            )
        }
        PackageManager::Brew => wsl::run_root_check("brew install postgresql"),
        other => anyhow::bail!("Unsupported package manager for PostgreSQL: {other:?}"),
    }
}

/// Start the cluster, enable the service on boot, and provision the
/// application role + database. Idempotent.
pub fn configure() -> anyhow::Result<()> {
    let user = std::env::var("DB_USER").unwrap_or_else(|_| crate::DB_USER.to_string());
    let password = std::env::var("DB_PASSWORD").unwrap_or_else(|_| crate::DB_PASSWORD.to_string());
    let db = std::env::var("DB_NAME").unwrap_or_else(|_| crate::DB_NAME.to_string());

    // 1. Start the cluster if it isn't online, and enable the service on boot.
    let cluster_online =
        "pg_lsclusters | awk 'NR>1 { if ($4 == \"online\") found=1 } END { exit !found }'";
    if !wsl::run_root_ok(cluster_online) {
        tracing::info!("Starting PostgreSQL cluster...");
        wsl::run_root_check("service postgresql start")?;
    }
    // Best-effort: only meaningful when systemd is the init system.
    let _ = wsl::run_root_ok("systemctl enable postgresql");

    // 2. Create the application role (idempotent).
    if !role_exists(&user) {
        tracing::info!("Creating role '{user}' (LOGIN CREATEDB)...");
        wsl::run_root_check(&format!(
            "su -s /bin/bash postgres -c \"psql -v ON_ERROR_STOP=1 -c \\\"CREATE ROLE {user} LOGIN CREATEDB PASSWORD '{password}';\\\"\""
        ))?;
    }

    // 3. Create the application database (idempotent).
    if !database_exists(&db) {
        tracing::info!("Creating database '{db}' (owner: {user})...");
        wsl::run_root_check(&format!(
            "su -s /bin/bash postgres -c \"psql -v ON_ERROR_STOP=1 -c \\\"CREATE DATABASE {db} OWNER {user};\\\"\""
        ))?;
    }

    // 4. Make sure the role owns the database going forward.
    wsl::run_root_check(&format!(
        "su -s /bin/bash postgres -c \"psql -v ON_ERROR_STOP=1 -c \\\"ALTER DATABASE {db} OWNER TO {user};\\\"\""
    ))?;

    Ok(())
}

fn role_exists(user: &str) -> bool {
    wsl::run_root_output(&format!(
        "su -s /bin/bash postgres -c \"psql -Atc \\\"SELECT 1 FROM pg_roles WHERE rolname='{user}'\\\"\""
    )) == "1"
}

fn database_exists(db: &str) -> bool {
    wsl::run_root_output(&format!(
        "su -s /bin/bash postgres -c \"psql -Atc \\\"SELECT 1 FROM pg_database WHERE datname='{db}'\\\"\""
    )) == "1"
}
