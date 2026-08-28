use once_cell::sync::OnceCell;
use sqlx::{
    postgres::PgPoolOptions,
    PgPool,
};

static DATABASE: OnceCell<PgPool> = OnceCell::new();

pub async fn connect(
    database_url: &str,
) -> anyhow::Result<()> {

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;

    DATABASE
        .set(pool)
        .map_err(|_| anyhow::anyhow!("Database already initialized"))?;

    Ok(())
}

pub fn pool() -> &'static PgPool {
    DATABASE
        .get()
        .expect("Database not initialized")
}