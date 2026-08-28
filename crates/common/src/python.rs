use pyo3::prelude::*;
use crate::{
    init_logging as rust_init_logging,
    log_info as rust_log_info,
    log_error as rust_log_error,
    log_debug as rust_log_debug,
    log_warn as rust_log_warn,
};


/// Initialise the tracing logging system.
/// 
/// This function can only be called once. Subsequent calls will be ignored.
/// 
/// # Arguments
/// 
/// `level` - Optional log level override. Pass `None` to use the `RUST_LOG`
/// environment variable, or on of `"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`.
/// 
/// # Examples
/// 
/// import entweni_common
/// 
/// # Use RUST_LOG from environment/.env
/// entweni_common.init_logging(None)
/// 
/// # Or specify level directly (Override)
/// entweni_common.init_logging("debug")
///
/// # Log Levels
///
/// Level    | Use Case
/// ---------|------------------------------------------
/// trace    | Very detailed logs, typically only in development
/// debug    | Diagnostic information for debugging
/// info     | General operational messages (default)
/// warn     | Unexpected behavior that is still handled
/// error    | Failures requiring attention


#[pyfunction]
fn init_logging(level: Option<&str>) {
    rust_init_logging(level);
}

#[pyfunction]
fn log_info(msg: &str) { rust_log_info(msg); }

/// Logging on the debug
#[pyfunction]
fn log_debug(msg: &str) {
    rust_log_debug(msg);
}

#[pyfunction]
fn log_warn(msg: &str) { rust_log_warn(msg); }

#[pyfunction]
fn log_error(msg: &str) { rust_log_error(msg); }

#[pymodule]
fn entweni_common(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init_logging, m)?)?;
    m.add_function(wrap_pyfunction!(log_info, m)?)?;
    m.add_function(wrap_pyfunction!(log_debug, m)?)?;
    m.add_function(wrap_pyfunction!(log_warn, m)?)?;
    m.add_function(wrap_pyfunction!(log_error, m)?)?;
    Ok(())
}