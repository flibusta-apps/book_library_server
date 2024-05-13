use crate::{config::CONFIG, prisma::PrismaClient};

pub async fn get_prisma_client() -> PrismaClient {
    let database_url: String = format!(
        "postgresql://{}:{}@{}:{}/{}?connection_limit=10",
        CONFIG.postgres_user,
        CONFIG.postgres_password,
        CONFIG.postgres_host,
        CONFIG.postgres_port,
        CONFIG.postgres_db
    );

    PrismaClient::_builder()
        .with_url(database_url)
        .build()
        .await
        .unwrap()
}
