//! Bridge for running system commands in the E-Lupane runtime environment.
//!
//! On Windows every system service (PostgreSQL, Redis) lives inside WSL, so all
//! commands are dispatched through `wsl -u root -e bash -lc "<cmd>"`. When the
//! installer itself runs inside Linux (e.g. directly in WSL) the command runs
//! through `bash -lc`, elevated with `sudo` when the current user is not root.

use std::process::{Command, Output};

/// Run a shell command in the E-Lupane environment and return its output.
pub fn run(cmd: &str) -> std::io::Result<Output> {
    #[cfg(target_os = "windows")]
    {
        Command::new("wsl")
            .args(["-u", "root", "-e", "bash", "-lc", cmd])
            .output()
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("bash").arg("-lc").arg(cmd).output()
    }
}

/// Run a command and report whether it exited successfully.
pub fn run_ok(cmd: &str) -> bool {
    run(cmd).map(|o| o.status.success()).unwrap_or(false)
}

/// Run a command and return its trimmed stdout (empty string on failure).
pub fn run_output(cmd: &str) -> String {
    run(cmd)
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Run a command and return stdout (or the stderr/error message if stdout is
/// empty). Used for diagnostics.
pub fn run_output_err(cmd: &str) -> String {
    run(cmd)
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            if !out.is_empty() {
                out
            } else if !err.is_empty() {
                err
            } else {
                "no output".to_string()
            }
        })
        .unwrap_or_else(|e| format!("runner error: {e}"))
}

/// Wrap a command so it executes as root in the target environment.
///
/// `wsl -u root` already runs as root inside the distro, so on Windows no
/// elevation prefix is needed. On native Linux we fall back to `sudo`.
fn root_cmd(cmd: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        cmd.to_string()
    }

    #[cfg(not(target_os = "windows"))]
    {
        if run_output("id -u").trim() == "0" {
            cmd.to_string()
        } else {
            format!("sudo {cmd}")
        }
    }
}

/// Run a command as root and return its raw output.
pub fn run_root(cmd: &str) -> std::io::Result<Output> {
    run(&root_cmd(cmd))
}

/// Run a command as root and report whether it exited successfully.
pub fn run_root_ok(cmd: &str) -> bool {
    run_ok(&root_cmd(cmd))
}

/// Run a command as root and return its trimmed stdout.
pub fn run_root_output(cmd: &str) -> String {
    run_output(&root_cmd(cmd))
}

/// Run a command as root and fail if it exits non-zero.
pub fn run_root_check(cmd: &str) -> anyhow::Result<()> {
    let output = run_root(cmd)?;
    if !output.status.success() {
        anyhow::bail!("command failed ({:?}): {cmd}", output.status.code());
    }
    Ok(())
}
