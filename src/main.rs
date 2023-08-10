pub mod config;
pub mod views;
pub mod prisma;
pub mod db;
pub mod serializers;
pub mod meilisearch;

use std::{net::SocketAddr, str::FromStr};
use sentry::{ClientOptions, types::Dsn, integrations::debug_images::DebugImagesIntegration};
use tracing::info;

use crate::views::get_router;


#[tokio::main]
async fn main() {
    let options = ClientOptions {
        dsn: Some(Dsn::from_str(&config::CONFIG.sentry_dsn).unwrap()),
        default_integrations: false,
        ..Default::default()
    }
    .add_integration(DebugImagesIntegration::new());

    let _guard = sentry::init(options);

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let app = get_router().await;

    info!("Start webserver...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    info!("Webserver shutdown...")
}
