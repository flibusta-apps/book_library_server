use once_cell::sync::Lazy;

fn get_env(env: &'static str) -> String {
    std::env::var(env).unwrap_or_else(|_| panic!("Cannot get the {} env variable", env))
}

fn get_env_opt(env: &'static str) -> Option<String> {
    std::env::var(env)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn get_env_or<T: std::str::FromStr>(env: &'static str, default: T) -> T {
    std::env::var(env)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

pub struct Config {
    pub api_key: String,

    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_host: String,
    pub postgres_port: u32,
    pub postgres_db: String,

    /// Max size of the PostgreSQL connection pool. Env: `POSTGRES_MAX_CONNECTIONS`. Default: 10.
    pub postgres_max_connections: u32,
    /// How long to wait for a free connection before failing. Env: `POSTGRES_ACQUIRE_TIMEOUT_SECS`. Default: 5.
    pub postgres_acquire_timeout_secs: u64,
    /// `statement_timeout` set on every new connection. Env: `POSTGRES_STATEMENT_TIMEOUT_SECS`. Default: 30.
    pub postgres_statement_timeout_secs: u64,

    pub meili_host: String,
    pub meili_master_key: String,
    /// HTTP timeout for the Meilisearch client. Env: `MEILI_HTTP_TIMEOUT_SECS`. Default: 10.
    pub meili_http_timeout_secs: u64,

    /// Optional Sentry DSN. When unset/empty, Sentry is not initialized.
    pub sentry_dsn: Option<String>,
}

impl Config {
    pub fn load() -> Config {
        Config {
            api_key: get_env("API_KEY"),

            postgres_user: get_env("POSTGRES_USER"),
            postgres_password: get_env("POSTGRES_PASSWORD"),
            postgres_host: get_env("POSTGRES_HOST"),
            postgres_port: get_env("POSTGRES_PORT")
                .parse()
                .expect("POSTGRES_PORT must be a valid u32"),
            postgres_db: get_env("POSTGRES_DB"),

            postgres_max_connections: get_env_or("POSTGRES_MAX_CONNECTIONS", 10),
            postgres_acquire_timeout_secs: get_env_or("POSTGRES_ACQUIRE_TIMEOUT_SECS", 5),
            postgres_statement_timeout_secs: get_env_or("POSTGRES_STATEMENT_TIMEOUT_SECS", 30),

            meili_host: get_env("MEILI_HOST"),
            meili_master_key: get_env("MEILI_MASTER_KEY"),
            meili_http_timeout_secs: get_env_or("MEILI_HTTP_TIMEOUT_SECS", 10),

            sentry_dsn: get_env_opt("SENTRY_DSN"),
        }
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::load);
