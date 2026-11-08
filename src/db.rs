use crate::config::CONFIG;

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};

pub async fn get_postgres_pool() -> PgPool {
    let mut options = PgConnectOptions::new()
        .host(&CONFIG.postgres_host)
        .port(CONFIG.postgres_port)
        .username(&CONFIG.postgres_user)
        .password(&CONFIG.postgres_password)
        .database(&CONFIG.postgres_db)
        .application_name("book_library_server");

    // Preserve SQLx's default when neither override is set.
    if let Ok(mode) = std::env::var("DATABASE_SSLMODE").or_else(|_| std::env::var("PGSSLMODE")) {
        options = options.ssl_mode(
            mode.parse::<PgSslMode>()
                .expect("Invalid PostgreSQL SSL mode"),
        );
    }

    let statement_timeout_secs = CONFIG.postgres_statement_timeout_secs;

    let pool = PgPoolOptions::new()
        .max_connections(CONFIG.postgres_max_connections)
        .acquire_timeout(std::time::Duration::from_secs(
            CONFIG.postgres_acquire_timeout_secs,
        ))
        .after_connect(move |conn, _meta| {
            Box::pin(async move {
                sqlx::query("SELECT set_config('statement_timeout', $1, false)")
                    .bind(format!("{statement_timeout_secs}s"))
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect_with(options)
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
