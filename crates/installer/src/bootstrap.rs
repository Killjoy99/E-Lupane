use crate::checks::{postgres, redis};
use crate::config;
use crate::package_manager;
use crate::report::{print_report, CheckResult};
use crate::{installers, package_manager::PackageManager};

/// Run only the readiness checks and print the report.
pub fn run_checks() {
    let manager = package_manager::detect_package_manager();
    tracing::info!("Detected package manager: {:?}", manager);

    let results = collect_results();
    print_report(&results);
}

/// Run the full install/configure flow (`--setup`, `--install`, `--configure`).
///
/// * `do_install`   — install missing components.
/// * `do_configure` — start/enable services, create the `elupane` role and
///   database, and write `DATABASE_URL` / `REDIS_URL` into the root `.env`.
pub fn run(do_install: bool, do_configure: bool) -> anyhow::Result<()> {
    let manager = package_manager::detect_package_manager();
    tracing::info!("Detected package manager: {:?}", manager);

    if do_install {
        install_step(manager)?;
    }

    if do_configure {
        configure_step()?;
    }

    run_checks();
    Ok(())
}

fn install_step(manager: PackageManager) -> anyhow::Result<()> {
    if postgres::get_status().installed {
        tracing::info!("PostgreSQL already installed — skipping install.");
    } else {
        tracing::info!("Installing PostgreSQL...");
        installers::postgres::install(manager)?;
    }

    if redis::get_status().installed {
        tracing::info!("Redis already installed — skipping install.");
    } else {
        tracing::info!("Installing Redis...");
        installers::redis::install(manager)?;
    }

    Ok(())
}

fn configure_step() -> anyhow::Result<()> {
    tracing::info!(
        "Configuring PostgreSQL: start service, enable on boot, create \
         '{}' role + '{}' database...",
        crate::DB_USER,
        crate::DB_NAME
    );
    installers::postgres::configure()?;

    tracing::info!("Configuring Redis: start service, enable on boot...");
    installers::redis::configure()?;

    tracing::info!("Writing DATABASE_URL + REDIS_URL into .env ...");
    config::generate::ensure_env()?;

    Ok(())
}

fn collect_results() -> Vec<CheckResult> {
    vec![
        postgres::check(),
        postgres::check_config(),
        redis::check(),
    ]
}
