use std::time::Duration;

use axum::{
    http::{self, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Extension, Router,
};
use axum_prometheus::PrometheusMetricLayer;
use once_cell::sync::Lazy;
use sqlx::PgPool;
use subtle::ConstantTimeEq;
use tower_http::{
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::{self, TraceLayer},
};
use tracing::Level;
use uuid::Uuid;

use crate::{config::CONFIG, db::get_postgres_pool, meilisearch::MEILI_CLIENT};

/// HTTP server-side request timeout. Slow clients / stalled upstreams get cut
/// off instead of tying up a task forever.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// Max accepted request body size (10 MiB).
const REQUEST_BODY_LIMIT: usize = 10 * 1024 * 1024;

#[derive(Clone, Default)]
struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        http::HeaderValue::from_str(&id).ok().map(RequestId::new)
    }
}

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

pub type Database = Extension<PgPool>;

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

    let is_valid: bool = auth_header
        .as_bytes()
        .ct_eq(CONFIG.api_key.as_bytes())
        .into();

    if !is_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}

/// Pure liveness probe: always returns 200 if the process is up and able to
/// respond, regardless of downstream dependency health.
async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Readiness probe: verifies the PostgreSQL pool can actually serve a query.
/// Returns 503 if the database is unreachable/exhausted so orchestrators can
/// take the instance out of rotation instead of routing traffic to it.
async fn ready_check(Extension(pool): Extension<PgPool>) -> StatusCode {
    match sqlx::query_scalar!("SELECT 1").fetch_one(&pool).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            tracing::error!(error = %err, "readiness check failed: database unreachable");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

pub async fn get_router() -> Router {
    let client = get_postgres_pool().await;

    // Touch the shared Meilisearch client once at startup so it's constructed
    // eagerly rather than lazily on first request.
    Lazy::force(&MEILI_CLIENT);

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app_router = Router::new()
        .nest("/api/v1/authors", get_authors_router().await)
        .nest("/api/v1/translators", get_translators_router().await)
        .nest("/api/v1/genres", get_genres_router().await)
        .nest("/api/v1/books", get_books_router().await)
        .nest("/api/v1/sequences", get_sequences_router().await)
        .layer(middleware::from_fn(auth))
        .layer(Extension(client.clone()))
        .layer(prometheus_layer);

    let health_router = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(ready_check))
        .layer(Extension(client));

    let metric_router = Router::new()
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(middleware::from_fn(auth));

    Router::new()
        .merge(app_router)
        .merge(health_router)
        .merge(metric_router)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TimeoutLayer::new(REQUEST_TIMEOUT))
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT))
}
