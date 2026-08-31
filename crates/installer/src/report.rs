use tracing::info;

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

impl CheckResult {
    pub fn success(name: &str, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: message.into(),
        }
    }

    pub fn failure(name: &str, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            message: message.into(),
        }
    }
}

pub fn print_report(results: &[CheckResult]) {
    info!("===============================");
    info!("     E-LUPANE INSTALLER");
    info!("===============================");

    for result in results {
        let status = if result.passed {
            "✓"
        } else {
            "✗"
        };

        info!(
            "[{}] {:15} {}",
            status,
            result.name,
            result.message
        );
    }
}