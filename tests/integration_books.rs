//! Integration tests for the `/books/*` handlers.
//!
//! Each test gets a fresh, isolated Postgres database (via `#[sqlx::test]`)
//! with all migrations from `./migrations` applied. Requires `DATABASE_URL`
//! to point at a reachable Postgres server the test runner is allowed to
//! create/drop scratch databases on (sqlx creates one throwaway DB per test).
//!
//! This test locks in two fixes from Spec 11.1 (see
//! docs/specs/11-duplication-dead-code.md#11.1):
//! - the consolidated `DetailBook` query (`query_detail_book!` macro) is used
//!   by `get_book`, and
//! - a NULL `authors.middle_name` no longer causes a 500 (`UnexpectedNull`),
//!   decoding instead as an empty string.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Extension, Router,
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

use book_library_server::views::books::get_books_router;

async fn app(pool: PgPool) -> Router {
    Router::new()
        .merge(get_books_router())
        .layer(Extension(pool))
}

#[sqlx::test(migrations = "./migrations")]
async fn get_book_returns_200_with_empty_string_for_null_middle_name(pool: PgPool) {
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
        "INSERT INTO book_authors (book, author) VALUES ($1, $2)",
        book_id,
        author_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let genre_id: i32 = sqlx::query_scalar!(
        r#"
        INSERT INTO genres (source, remote_id, code, description, meta)
        VALUES (1, 1, 'test_code', 'Test genre', 'test_meta')
        RETURNING id
        "#
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO book_genres (book, genre) VALUES ($1, $2)",
        book_id,
        genre_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = app(pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/{book_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["id"], book_id);
    assert_eq!(json["title"], "Test Book");
    assert_eq!(json["authors"][0]["id"], author_id);
    assert_eq!(json["authors"][0]["first_name"], "Ivan");
    assert_eq!(json["authors"][0]["last_name"], "Ivanov");
    assert_eq!(
        json["authors"][0]["middle_name"], "",
        "NULL middle_name must decode as an empty string, not 500"
    );
    assert_eq!(json["genres"][0]["id"], genre_id);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_book_returns_404_for_missing_book(pool: PgPool) {
    let app = app(pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/999999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
