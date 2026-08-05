//! Integration tests for the `/translators/*` handlers.
//!
//! Each test gets a fresh, isolated Postgres database (via `#[sqlx::test]`)
//! with all migrations from `./migrations` applied. Requires `DATABASE_URL`
//! to point at a reachable Postgres server the test runner is allowed to
//! create/drop scratch databases on (sqlx creates one throwaway DB per test).
//!
//! Handlers are exercised through the real `translators` sub-router (built
//! via `get_translators_router()`) driven with `tower::ServiceExt::oneshot`,
//! rather than the full app router, since these handlers don't depend on
//! `CONFIG`/auth -- this keeps the test independent of process-wide env vars.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Extension, Router,
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

use book_library_server::views::translators::get_translators_router;

async fn app(pool: PgPool) -> Router {
    Router::new()
        .merge(get_translators_router())
        .layer(Extension(pool))
}

#[sqlx::test(migrations = "./migrations")]
async fn get_translated_books_returns_404_for_missing_translator(pool: PgPool) {
    let app = app(pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/999999/books")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_translated_books_returns_translator_and_books(pool: PgPool) {
    sqlx::query!("INSERT INTO sources (id, name) VALUES (1, 'test-source')")
        .execute(&pool)
        .await
        .unwrap();

    let author_id: i32 = sqlx::query_scalar!(
        r#"
        INSERT INTO authors (source, remote_id, first_name, last_name, middle_name)
        VALUES (1, 1, 'Ivan', 'Ivanov', NULL)
        RETURNING id
        "#
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let book_id: i32 = sqlx::query_scalar!(
        r#"
        INSERT INTO books (source, remote_id, title, lang, file_type, uploaded, is_deleted, year)
        VALUES (1, 1, 'Test Book', 'ru', 'fb2', CURRENT_DATE, false, 2020)
        RETURNING id
        "#
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO translations (position, author, book) VALUES (1, $1, $2)",
        author_id,
        book_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = app(pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/{author_id}/books"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["parent_item"]["id"], author_id);
    assert_eq!(json["parent_item"]["first_name"], "Ivan");
    assert_eq!(json["parent_item"]["last_name"], "Ivanov");
    assert_eq!(json["total"], 1);
    assert_eq!(json["items"][0]["id"], book_id);
    assert_eq!(json["items"][0]["title"], "Test Book");
}
