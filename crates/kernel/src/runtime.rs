use anyhow::Result;

use crate::bootstrap::bootstrap;

pub async fn run() -> Result<()> {
    let state = bootstrap().await?;

    tracing::info!("Kernel started");

    // List all the loaded plugins
    let plugins = state.plugin_manager.list_plugins().await;
    tracing::info!("Loaded plugins: {:?}", plugins);

    let child_count = state.plugin_children.len();
    if child_count == 0 {
        tracing::warn!("No plugins were launched. Kernel supervisor will idle until Ctrl+C.");
    }

    tracing::info!(
        "Kernel supervisor active — {} plugin process(es) running. Press Ctrl+C to shutdown.",
        child_count
    );

    // Keep the supervisor process alive until a shutdown signal is received.
    // Without this the gateway would exit immediately after bootstrap, orphaning
    // all child plugin processes.
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutdown signal received — terminating child processes...");

    // Gracefully terminate every child plugin process.
    for mut child in state.plugin_children {
        let _ = child.kill().await;
    }

    tracing::info!("All child processes terminated. Goodbye.");
    Ok(())
}
