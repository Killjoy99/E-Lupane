use crate::checks::{postgres, redis};
use crate::report::{CheckResult, print_report};

pub fn run_checks() {
    let results: Vec<CheckResult> = vec![
        postgres::check(),
        redis::check(),
    ];

    print_report(&results);
}