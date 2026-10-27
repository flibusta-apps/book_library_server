//! Full-app router test: exercises the real `get_router()` (built exactly as
//! `main.rs` does) to assert the auth policy from
//! docs/specs/02-*-auth*.md / 02.1:
//!
//! - `/api/v1/*` requires a matching `Authorization` header (401 without it,
//!   passes through with it).
//! - `/metrics` requires auth too.
//! - `/health` does NOT require auth.
//!
//! This test needs `CONFIG` (a process-wide `Lazy`) to be initialized with a
//! specific set of env vars *before* anything in the crate touches it, so it
//! lives in its own test binary (every file under `tests/` compiles to a
//! separate binary, so `CONFIG`'s `Lazy` is fresh here). It requires a real,
//! reachable Postgres (pointed at via `POSTGRES_*` env vars, matching
//! `src/config.rs`) since `get_router()` calls `get_postgres_pool()`, which
//! connects and runs `sqlx::migrate!` itself. `MEILI_HOST`/`MEILI_MASTER_KEY`
//! only need to be *set* (to anything) -- the Meilisearch client is
//! constructed lazily and isn't dialed until a search endpoint is hit, which
//! this test doesn't do.
//!
//! Env vars this test sets on the process (documented for the CI lane that
//! needs to provide a `postgres:` service):
//! `API_KEY`, `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_HOST`,
//! `POSTGRES_PORT`, `POSTGRES_DB`, `MEILI_HOST`, `MEILI_MASTER_KEY`.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use tower::ServiceExt;

const TEST_API_KEY: &str = "test-api-key-for-router-auth-test";

const DEFAULT_DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5433/routertest";

/// Splits `postgres://user:password@host:port/db` into its parts.
fn parse_database_url(url: &str) -> Option<(String, String, String, String, String)> {
    let rest = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))?;
    let (creds, rest) = rest.split_once('@')?;
    let (user, password) = creds.split_once(':').unwrap_or((creds, ""));
    let (hostport, db) = rest.split_once('/')?;
    let db = db.split('?').next()?;
    let (host, port) = hostport.split_once(':').unwrap_or((hostport, "5432"));
    Some((
        user.to_string(),
        password.to_string(),
        host.to_string(),
        port.to_string(),
        db.to_string(),
    ))
}

fn set_test_env() {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let (user, password, host, port, db) = parse_database_url(&url)
        .or_else(|| parse_database_url(DEFAULT_DATABASE_URL))
        .expect("valid database url");
    // SAFETY: this test binary's `main` is single-threaded test harness
    // startup; these are set once, before `CONFIG` (a `Lazy`) is first
    // touched by any test in this binary.
    unsafe {
        std::env::set_var("API_KEY", TEST_API_KEY);
        std::env::set_var("POSTGRES_USER", user);
        std::env::set_var("POSTGRES_PASSWORD", password);
        std::env::set_var("POSTGRES_HOST", host);
        std::env::set_var("POSTGRES_PORT", port);
        std::env::set_var("POSTGRES_DB", db);
        std::env::set_var("MEILI_HOST", "http://localhost:7700");
        std::env::set_var("MEILI_MASTER_KEY", "test-meili-key");
    }
}

// NOTE: all four assertions live in a single test function (rather than four
// separate `#[tokio::test]`s) because `get_router()` calls
// `PrometheusMetricLayer::pair()`, which registers a *process-wide* global
// metrics recorder exactly once; calling `get_router()` more than once per
// process panics with "Failed to set global recorder". `Router` is `Clone`,
// so we build it once and clone it per `oneshot` call below.
#[tokio::test]
async fn router_auth_policy() {
    set_test_env();
    let app = book_library_server::views::get_router().await;

    // `/api/v1/*` with no `Authorization` header -> 401.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/books")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected /api/v1/books without auth to be rejected"
    );

    // `/api/v1/*` with the correct `Authorization` header -> passes auth
    // (whatever status the handler itself returns is fine; just not 401).
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/books")
                .header(header::AUTHORIZATION, TEST_API_KEY)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected /api/v1/books with correct auth to succeed"
    );

    // C3: both `/api/v1/books/base` and `/api/v1/books/base/` must be routed
    // (never 404/405) -- with auth they reach the handler.
    for uri in ["/api/v1/books/base", "/api/v1/books/base/"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .header(header::AUTHORIZATION, TEST_API_KEY)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "expected {uri} to be served"
        );
    }

    // `/metrics` with no `Authorization` header -> 401 (Spec 02.1).
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected /metrics without auth to be rejected"
    );

    // `/health` with no `Authorization` header -> 200 (must not require auth).
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected /health without auth to succeed"
    );
}
