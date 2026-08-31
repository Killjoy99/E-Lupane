//! Installer state + entry point used by the rest of the workspace.
//!
//! `InstallerState` tracks what a completed run has provisioned so callers can
//! decide whether more work is required.

use crate::bootstrap;

#[derive(Debug, Default)]
pub struct InstallerState {
    pub postgres_installed: bool,
    pub redis_installed: bool,
    pub database_created: bool,
    pub migrations_applied: bool,
    pub config_generated: bool,
}

/// Convenience wrapper around [`bootstrap::run`] — full setup
/// (install what is missing, then configure).
pub fn run_installer() -> anyhow::Result<()> {
    bootstrap::run(true, true)
}