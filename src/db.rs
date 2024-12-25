use crate::config::CONFIG;

use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn get_postgres_pool() -> PgPool {
    let database_url: String = format!(
        "postgresql://{}:{}@{}:{}/{}",
        CONFIG.postgres_user,
        CONFIG.postgres_password,
        CONFIG.postgres_host,
        CONFIG.postgres_port,
        CONFIG.postgres_db
    );

    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(300))
        .connect(&database_url)
        .await
        .unwrap()
}
