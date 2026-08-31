use crate::package_manager::{effective_manager, PackageManager};
use crate::wsl;

/// Install Redis in the E-Lupane runtime environment (WSL on Windows, the
/// local distro on Linux).
pub fn install(manager: PackageManager) -> anyhow::Result<()> {
    match effective_manager(manager) {
        PackageManager::Apt => {
            wsl::run_root_check("apt-get update")?;
            wsl::run_root_check("DEBIAN_FRONTEND=noninteractive apt-get install -y redis-server")
        }
        PackageManager::Brew => wsl::run_root_check("brew install redis"),
        other => anyhow::bail!("Unsupported package manager for Redis: {other:?}"),
    }
}

/// Start the Redis service and enable it on boot. Idempotent.
pub fn configure() -> anyhow::Result<()> {
    if !wsl::run_ok("redis-cli -h 127.0.0.1 ping | grep -q PONG") {
        tracing::info!("Starting Redis...");
        wsl::run_root_check("service redis-server start")?;
    }
    // Best-effort: only meaningful when systemd is the init system.
    let _ = wsl::run_root_ok("systemctl enable redis-server");
    Ok(())
}