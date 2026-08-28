use tracing::info;

mod report;
mod package_manager;
mod checks{
    pub mod postgres;
    pub mod redis;
}

use report::{print_report, CheckResult};

fn main() {
    // Load the .env
    dotenvy::dotenv().ok();
    // Initialise Tracing
    common::init_logging(None);

    let mut results = Vec::new();
    results.push(checks::postgres::check());
    results.push(checks::redis::check());

    print_report(&results);

    info!("Detected package manager: {:?}",
        package_manager::detect_package_manager());
}