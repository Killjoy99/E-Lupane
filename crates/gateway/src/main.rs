use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load the .env
    dotenvy::dotenv().ok();
    // Initialise Tracing
    common::init_logging(None);

    let _ =kernel::runtime::run().await;

    Ok(())
}