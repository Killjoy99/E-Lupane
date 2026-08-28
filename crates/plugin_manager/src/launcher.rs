use std::process::Stdio;

use anyhow::Result;

use tokio::process::{Child, Command};

use elupane_core::manifest::{
    PluginLanguage,
    PluginManifest,
};

/// Launch a plugin as a child process.
///
/// Returns the PID and the live [`Child`] handle so the caller (the kernel
/// supervisor) can keep the process group alive and terminate children on
/// shutdown.
pub async fn launch_plugin(
    manifest: &PluginManifest,
) -> Result<(u32, Child)> {
    let mut command = match manifest.language {
        PluginLanguage::Rust => {
            Command::new(&manifest.entry)
        }

        PluginLanguage::Python => {
            let mut cmd = Command::new("python");

            cmd.arg(&manifest.entry);

            cmd
        }

        PluginLanguage::Php => {
            let mut cmd = Command::new("php");
            cmd.arg(&manifest.entry);

            cmd
        }

        _ => {
            anyhow::bail!(
                "Unsupported plugin language"
            );
        }
    };

    // If the manifest specifies a working directory, launch the plugin from
    // there so it can find its own .env, assets, etc.
    if let Some(dir) = &manifest.working_dir {
        command.current_dir(dir);
    }

    let child = command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let pid = child.id().unwrap_or_default();

    tracing::info!(
        "Started plugin {} with PID {}",
        manifest.name,
        pid
    );

    Ok((pid, child))
}
