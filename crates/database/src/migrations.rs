pub async fn migrate_core() -> anyhow::Result<()> {

    sqlx::migrate!("../migrations")
        .run(pool())
        .await?;

    Ok(())
}