use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_subscriber::fmt::time::OffsetTime;
use time::macros::format_description;
use std::sync::Once;

static INIT: Once = Once::new();

#[cfg(any(feature="init", feature="python"))]
pub fn init_logging(level: Option<&str>) {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level.unwrap_or("info")));

        let time_format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

        // OffsetTime::new returns OffsetTime directly, not Result
        let timer = OffsetTime::new(time::UtcOffset::UTC, time_format);

        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().with_timer(timer))
            .init();
    });
}

pub fn log_info(msg: &str) {
    tracing::info!("{}", msg);
}

pub fn log_error(msg: &str) {
    tracing::error!("{}", msg);
}

pub fn log_debug(msg: &str) {
    tracing::debug!("{}", msg);
}

pub fn log_warn(msg: &str) {
    tracing::warn!("{}", msg);
}

#[cfg(feature = "python")]
mod python;