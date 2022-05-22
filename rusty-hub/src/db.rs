use sqlx;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

pub async fn prepare_from_env() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::migrate!().run(&pool).await.unwrap();
    Ok(pool)
}
