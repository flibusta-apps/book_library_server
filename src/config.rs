use once_cell::sync::Lazy;

fn get_env(env: &'static str) -> String {
    std::env::var(env).unwrap_or_else(|_| panic!("Cannot get the {} env variable", env))
}

pub struct Config {
    pub api_key: String,

    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_host: String,
    pub postgres_port: u32,
    pub postgres_db: String,

    pub meili_host: String,
    pub meili_master_key: String,

    pub sentry_dsn: String,
}

impl Config {
    pub fn load() -> Config {
        Config {
            api_key: get_env("API_KEY"),

            postgres_user: get_env("POSTGRES_USER"),
            postgres_password: get_env("POSTGRES_PASSWORD"),
            postgres_host: get_env("POSTGRES_HOST"),
            postgres_port: get_env("POSTGRES_PORT").parse().unwrap(),
            postgres_db: get_env("POSTGRES_DB"),

            meili_host: get_env("MEILI_HOST"),
            meili_master_key: get_env("MEILI_MASTER_KEY"),

            sentry_dsn: get_env("SENTRY_DSN"),
        }
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::load);
