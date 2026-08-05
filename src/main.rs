pub mod config;
pub mod db;
pub mod error;
pub mod meilisearch;
pub mod serializers;
pub mod views;

use sentry::{integrations::debug_images::DebugImagesIntegration, types::Dsn, ClientOptions};
use sentry_tracing::EventFilter;
use std::{net::SocketAddr, str::FromStr};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::views::get_router;

#[tokio::main]
async fn main() {
    // Sentry is optional: when `SENTRY_DSN` is unset/empty the service still
    // starts (useful for local dev/tests), it just doesn't report to Sentry.
    let _guard = config::CONFIG.sentry_dsn.as_ref().map(|dsn| {
        let options = ClientOptions {
            dsn: Some(Dsn::from_str(dsn).expect("SENTRY_DSN is not a valid DSN")),
            // Keep default integrations enabled (PanicIntegration,
            // AttachStacktraceIntegration, ContextIntegration, ...) so panics
            // are actually captured, on top of DebugImagesIntegration.
            ..Default::default()
        }
        .add_integration(DebugImagesIntegration::new());

        sentry::init(options)
    });

    let sentry_layer = sentry_tracing::layer().event_filter(|md| match md.level() {
        &tracing::Level::ERROR => EventFilter::Event,
        _ => EventFilter::Ignore,
    });

    // `RUST_LOG` (e.g. `RUST_LOG=debug`) controls verbosity at runtime;
    // defaults to `info` when unset/invalid.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(env_filter)
        .with(sentry_layer)
        .init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let app = get_router().await;

    info!("Start webserver...");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind webserver address");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Webserver crashed");
    info!("Webserver shutdown...")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, waiting for in-flight requests to finish...");
}
