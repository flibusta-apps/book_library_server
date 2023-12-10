use std::sync::Arc;

use axum::{
    http::{self, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Extension, Router,
};
use axum_prometheus::PrometheusMetricLayer;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use crate::{config::CONFIG, db::get_prisma_client, prisma::PrismaClient};

use self::translators::get_translators_router;
use self::{
    authors::get_authors_router, books::get_books_router, genres::get_genres_router,
    sequences::get_sequences_router,
};

pub mod authors;
pub mod books;
pub mod common;
pub mod genres;
pub mod sequences;
pub mod translators;

pub type Database = Extension<Arc<PrismaClient>>;

async fn auth(req: Request<axum::body::Body>, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if auth_header != CONFIG.api_key {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}

pub async fn get_router() -> Router {
    let client = Arc::new(get_prisma_client().await);

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app_router = Router::new()
        .nest("/api/v1/authors", get_authors_router().await)
        .nest("/api/v1/translators", get_translators_router().await)
        .nest("/api/v1/genres", get_genres_router().await)
        .nest("/api/v1/books", get_books_router().await)
        .nest("/api/v1/sequences", get_sequences_router().await)
        .layer(middleware::from_fn(auth))
        .layer(Extension(client))
        .layer(prometheus_layer);

    let metric_router =
        Router::new().route("/metrics", get(|| async move { metric_handle.render() }));

    Router::new()
        .nest("/", app_router)
        .nest("/", metric_router)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}
