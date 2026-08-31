mod report;
mod package_manager;
mod wsl;
mod bootstrap;
mod installer_state;

pub mod checks {
    pub mod os;
    pub mod postgres;
    pub mod redis;
    pub mod utils;
}

pub mod installers {
    pub mod postgres;
    pub mod redis;
}

pub mod config {
    pub mod generate;
}

/// Default application credentials. Overridable at runtime via the matching
/// environment variables (see USAGE above).
pub const DB_USER: &str = "elupane";
pub const DB_PASSWORD: &str = "elupane";
pub const DB_NAME: &str = "elupane";
pub const DB_PORT: u16 = 5432;

const USAGE: &str = "\
E-Lupane installer — provisions PostgreSQL and Redis for the E-Lupane system.

On Windows, PostgreSQL and Redis run inside WSL; every command below is
dispatched into the WSL distro (as root) automatically.

USAGE:
  installer                Check PostgreSQL + Redis and print a report.
  installer --setup        Full setup: install what is missing, start &
                           enable services, create the elupane role/database,
                           and write DATABASE_URL + REDIS_URL to .env.
  installer --install      Install missing components only.
  installer --configure    Configure existing components only (start services,
                           create role/database, write .env).
  installer -h | --help    Show this help.

Environment variables (defaults in parentheses):
  DB_USER (elupane)        PostgreSQL role to create.
  DB_PASSWORD (elupane)    Password for that role.
  DB_NAME (elupane)        Database to create.
  DB_PORT (5432)           PostgreSQL port.
";

fn main() {
    // Load the .env
    dotenvy::dotenv().ok();
    // Initialise Tracing
    common::init_logging(None);

    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return;
    }

    let setup = args.iter().any(|a| matches!(a.as_str(), "--setup" | "--all" | "-s"));
    let install = args.iter().any(|a| matches!(a.as_str(), "--install" | "-i"));
    let configure = args.iter().any(|a| matches!(a.as_str(), "--configure" | "-c"));

    let result = if setup || install || configure {
        // `--setup` implies both install + configure; the narrower flags
        // enable exactly one of the two steps.
        bootstrap::run(setup || install, setup || configure)
    } else {
        bootstrap::run_checks();
        Ok(())
    };

    if let Err(err) = result {
        tracing::error!("Installer failed: {err:#}");
        std::process::exit(1);
    }
}
