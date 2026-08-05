use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Service-wide error type for handlers.
///
/// - `Db` / `Meili` are unexpected failures talking to external services: they are
///   logged (so Sentry/tracing capture the real cause) and turned into a generic
///   500 response, keeping the process alive instead of panicking.
/// - `NotFound` is an expected, user-facing condition (e.g. requested row missing).
#[derive(Debug)]
pub enum ApiError {
    Db(sqlx::Error),
    Meili(meilisearch_sdk::errors::Error),
    /// A Meilisearch request was aborted after exceeding `meilisearch::MEILI_TIMEOUT`
    /// (Spec 12.4) — e.g. a hung/blackholed Meilisearch instance — instead of
    /// hanging the handler indefinitely.
    MeiliTimeout,
    NotFound,
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::Db(err)
    }
}

impl From<meilisearch_sdk::errors::Error> for ApiError {
    fn from(err: meilisearch_sdk::errors::Error) -> Self {
        ApiError::Meili(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => StatusCode::NOT_FOUND.into_response(),
            ApiError::Db(err) => {
                tracing::error!(error = %err, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal server error" })),
                )
                    .into_response()
            }
            ApiError::Meili(err) => {
                tracing::error!(error = %err, "meilisearch error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal server error" })),
                )
                    .into_response()
            }
            ApiError::MeiliTimeout => {
                tracing::error!("meilisearch request timed out");
                (
                    StatusCode::GATEWAY_TIMEOUT,
                    Json(json!({ "error": "upstream search timed out" })),
                )
                    .into_response()
            }
        }
    }
}
