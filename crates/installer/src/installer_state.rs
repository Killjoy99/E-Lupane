// Installer state

#[derive(Debug)]
pub struct InstallerState {
    pub postgres_installed: bool,
    pub redis_installed: bool,
    pub database_created: bool,
    pub migrations_applied: bool,
    pub config_generated: bool,
}

pub async fn run_installer() -> anyhow::Result<()>{
    bootstrap::run().await
}