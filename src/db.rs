use crate::config::CONFIG;

use sqlx::{postgres::PgPoolOptions, Executor, PgPool};

pub async fn get_postgres_pool() -> PgPool {
    let database_url: String = format!(
        "postgresql://{}:{}@{}:{}/{}",
        CONFIG.postgres_user,
        CONFIG.postgres_password,
        CONFIG.postgres_host,
        CONFIG.postgres_port,
        CONFIG.postgres_db
    );

    let statement_timeout_secs = CONFIG.postgres_statement_timeout_secs;

    let pool = PgPoolOptions::new()
        .max_connections(CONFIG.postgres_max_connections)
        .acquire_timeout(std::time::Duration::from_secs(
            CONFIG.postgres_acquire_timeout_secs,
        ))
        .after_connect(move |conn, _meta| {
            Box::pin(async move {
                conn.execute(
                    format!("SET statement_timeout = '{}s'", statement_timeout_secs).as_str(),
                )
                .await?;
                Ok(())
            })
        })
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    // NOTE: every replica runs migrations on boot. sqlx uses a Postgres advisory
    // lock internally so concurrent `migrate!` runs across replicas are safe from
    // corrupting the `_sqlx_migrations` table, but it does mean multiple replicas
    // racing to acquire the lock at once during a rolling deploy. If this becomes
    // a problem, move migrations to a dedicated release/init step instead of
    // running them from every replica's startup path.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}
