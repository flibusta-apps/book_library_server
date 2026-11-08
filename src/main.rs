use book_library_server::{config, views::get_router};
use sentry::{integrations::debug_images::DebugImagesIntegration, ClientOptions};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{
    filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

fn sentry_error_layer<S>() -> impl Layer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    // Preserve the legacy ERROR-only event filter; do not turn lower levels
    // into Sentry errors, breadcrumbs, or structured logs.
    sentry_tracing::ErrorLayer::new().with_filter(LevelFilter::ERROR)
}

#[tokio::main]
async fn main() {
    // Sentry is optional: when `SENTRY_DSN` is unset/empty the service still
    // starts (useful for local dev/tests), it just doesn't report to Sentry.
    let _guard = config::CONFIG.sentry_dsn.as_ref().map(|dsn| {
        let options = ClientOptions::new()
            .dsn(dsn)
            // Keep default integrations enabled (PanicIntegration,
            // AttachStacktraceIntegration, ContextIntegration, ...) so panics
            // are actually captured, on top of DebugImagesIntegration.
            .add_integration(DebugImagesIntegration::new());

        sentry::init(options)
    });

    // `RUST_LOG` (e.g. `RUST_LOG=debug`) controls verbosity at runtime;
    // defaults to `info` when unset/invalid.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(env_filter)
        .with(sentry_tracing::SpanLayer::new().with_filter(LevelFilter::INFO))
        .with(sentry_error_layer())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentry_captures_only_errors_without_breadcrumbs() {
        let events = sentry::test::with_captured_events(|| {
            let subscriber = tracing_subscriber::registry()
                .with(sentry_tracing::SpanLayer::new().with_filter(LevelFilter::INFO))
                .with(sentry_error_layer());
            tracing::subscriber::with_default(subscriber, || {
                let _span = tracing::info_span!("request").entered();
                tracing::info!("not a Sentry event");
                tracing::warn!("not a breadcrumb");
                tracing::error!("dependency migration regression");
            });
        });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, sentry::Level::Error);
        assert_eq!(
            events[0].message.as_deref(),
            Some("dependency migration regression")
        );
        assert!(events[0].breadcrumbs.is_empty());
    }
}
