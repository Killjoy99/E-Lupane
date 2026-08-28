use sqlx::Row;

pub async fn ping() -> anyhow::Result<()> {

    sqlx::query("SELECT 1")
        .fetch_one(super::connection::pool())
        .await?;

    Ok(())
}