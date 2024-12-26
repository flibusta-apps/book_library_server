use std::collections::HashSet;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::{
    meilisearch::{get_meili_client, AuthorMeili},
    serializers::{
        allowed_langs::AllowedLangs,
        author::{Author, AuthorBook},
        author_annotation::AuthorAnnotation,
        book::BaseBook,
        pagination::{Page, PageWithParent, Pagination},
        sequence::Sequence,
    },
};

use super::{common::get_random_item::get_random_item, Database};

async fn get_authors(db: Database, pagination: Query<Pagination>) -> impl IntoResponse {
    let authors_count = sqlx::query_scalar!("SELECT COUNT(*) FROM authors",)
        .fetch_one(&db.0)
        .await
        .unwrap()
        .unwrap();

    let authors = sqlx::query_as!(
        Author,
        r#"
        SELECT
            a.id,
            a.first_name,
            a.last_name,
            COALESCE(a.middle_name, '') AS "middle_name!: String",
            CASE
                WHEN aa.id IS NOT NULL THEN true
                ELSE false
            END AS "annotation_exists!: bool"
        FROM authors a
        LEFT JOIN author_annotations aa ON a.id = aa.author
        ORDER BY a.id ASC
        OFFSET $1
        LIMIT $2
        "#,
        (pagination.page - 1) * pagination.size,
        pagination.size
    )
    .fetch_all(&db.0)
    .await
    .unwrap();

    let page: Page<Author> = Page::new(authors, authors_count, &pagination);

    Json(page)
}

async fn get_random_author(
    db: Database,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
) -> impl IntoResponse {
    let author_id = {
        let client = get_meili_client();

        let authors_index = client.index("authors");

        let filter = format!("author_langs IN [{}]", allowed_langs.join(", "));

        get_random_item::<AuthorMeili>(authors_index, filter).await
    };

    let author = sqlx::query_as!(
        Author,
        r#"
        SELECT
            a.id,
            a.first_name,
            a.last_name,
            COALESCE(a.middle_name, '') AS "middle_name!: String",
            CASE
                WHEN aa.id IS NOT NULL THEN true
                ELSE false
            END AS "annotation_exists!: bool"
        FROM authors a
        LEFT JOIN author_annotations aa ON a.id = aa.author
        WHERE a.id = $1
        "#,
        author_id
    )
    .fetch_one(&db.0)
    .await
    .unwrap();

    Json::<Author>(author)
}

async fn get_author(db: Database, Path(author_id): Path<i32>) -> impl IntoResponse {
    let author = sqlx::query_as!(
        Author,
        r#"
        SELECT
            a.id,
            a.first_name,
            a.last_name,
            COALESCE(a.middle_name, '') AS "middle_name!: String",
            CASE
                WHEN aa.id IS NOT NULL THEN true
                ELSE false
            END AS "annotation_exists!: bool"
        FROM authors a
        LEFT JOIN author_annotations aa ON a.id = aa.author
        WHERE a.id = $1
        "#,
        author_id
    )
    .fetch_optional(&db.0)
    .await
    .unwrap();

    match author {
        Some(author) => Json::<Author>(author).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_author_annotation(db: Database, Path(author_id): Path<i32>) -> impl IntoResponse {
    let author_annotation = sqlx::query_as!(
        AuthorAnnotation,
        r#"
        SELECT
            aa.id,
            aa.title,
            aa.text,
            aa.file
        FROM author_annotations aa
        WHERE aa.author = $1
        "#,
        author_id
    )
    .fetch_optional(&db.0)
    .await
    .unwrap();

    match author_annotation {
        Some(annotation) => Json::<AuthorAnnotation>(annotation).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_author_books(
    db: Database,
    Path(author_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let author = sqlx::query_as!(
        Author,
        r#"
        SELECT
            a.id,
            a.first_name,
            a.last_name,
            COALESCE(a.middle_name, '') AS "middle_name!: String",
            CASE
                WHEN aa.id IS NOT NULL THEN true
                ELSE false
            END AS "annotation_exists!: bool"
        FROM authors a
        LEFT JOIN author_annotations aa ON a.id = aa.author
        WHERE a.id = $1
        "#,
        author_id
    )
    .fetch_optional(&db.0)
    .await
    .unwrap();

    let author = match author {
        Some(author) => author,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let books_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM books b
        JOIN book_authors ba ON b.id = ba.book
        WHERE b.is_deleted = false AND ba.author = $1 AND b.lang = ANY($2)
        "#,
        author_id,
        &allowed_langs
    )
    .fetch_one(&db.0)
    .await
    .unwrap()
    .unwrap();

    let books = sqlx::query_as!(
        AuthorBook,
        r#"
        SELECT
            b.id,
            b.title,
            b.lang,
            b.file_type,
            b.year,
            CASE WHEN b.file_type = 'fb2' THEN ARRAY['fb2', 'epub', 'mobi', 'fb2zip']::text[] ELSE ARRAY[b.file_type]::text[] END AS "available_types!: Vec<String>",
            b.uploaded,
            (
                SELECT
                    ARRAY_AGG(
                        ROW(
                            authors.id,
                            authors.first_name,
                            authors.last_name,
                            authors.middle_name,
                            EXISTS(
                                SELECT * FROM author_annotations WHERE author = authors.id
                            )
                        )::author_type
                    )
                FROM translations
                JOIN authors ON authors.id = translations.author
                WHERE translations.book = b.id
            ) AS "translators!: Vec<Author>",
            (
                SELECT
                    ARRAY_AGG(
                        ROW(
                            sequences.id,
                            sequences.name
                        )::sequence_type
                    )
                FROM book_sequences
                JOIN sequences ON sequences.id = book_sequences.sequence
                WHERE book_sequences.book = b.id
            ) AS "sequences!: Vec<Sequence>",
            EXISTS(
                SELECT * FROM book_annotations WHERE book = b.id
            ) AS "annotation_exists!: bool"
        FROM books b
        JOIN book_authors ba ON b.id = ba.book
        WHERE b.is_deleted = false AND ba.author = $1 AND b.lang = ANY($2)
        ORDER BY b.title ASC
        OFFSET $3
        LIMIT $4
        "#,
        author_id,
        &allowed_langs,
        (pagination.page - 1) * pagination.size,
        pagination.size
    )
        .fetch_all(&db.0)
        .await
        .unwrap();

    let page: PageWithParent<AuthorBook, Author> =
        PageWithParent::new(author, books, books_count, &pagination);

    Json(page).into_response()
}

async fn get_author_books_available_types(
    db: Database,
    Path(author_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
) -> impl IntoResponse {
    // TODO: refactor

    let books = sqlx::query_as!(
        BaseBook,
        r#"
        SELECT
            b.id,
            CASE WHEN b.file_type = 'fb2' THEN ARRAY['fb2', 'epub', 'mobi', 'fb2zip']::text[] ELSE ARRAY[b.file_type]::text[] END AS "available_types!: Vec<String>"
        FROM books b
        JOIN book_authors ba ON b.id = ba.book
        WHERE b.is_deleted = false AND ba.author = $1 AND b.lang = ANY($2)
        "#,
        author_id,
        &allowed_langs
    )
        .fetch_all(&db.0)
        .await
        .unwrap();

    let mut file_types: HashSet<String> = HashSet::new();

    for book in books {
        for file_type in book.available_types {
            file_types.insert(file_type);
        }
    }

    Json::<Vec<String>>(file_types.into_iter().collect())
}

async fn search_authors(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let client = get_meili_client();

    let authors_index = client.index("authors");

    let filter = format!("author_langs IN [{}]", allowed_langs.join(", "));

    let result = authors_index
        .search()
        .with_query(&query)
        .with_filter(&filter)
        .with_offset(
            ((pagination.page - 1) * pagination.size)
                .try_into()
                .unwrap(),
        )
        .with_limit(pagination.size.try_into().unwrap())
        .execute::<AuthorMeili>()
        .await
        .unwrap();

    let total = result.estimated_total_hits.unwrap();
    let author_ids: Vec<i32> = result.hits.iter().map(|a| a.result.id).collect();

    let mut authors = sqlx::query_as!(
        Author,
        r#"
        SELECT
            a.id,
            a.first_name,
            a.last_name,
            COALESCE(a.middle_name, '') AS "middle_name!: String",
            CASE
                WHEN aa.id IS NOT NULL THEN true
                ELSE false
            END AS "annotation_exists!: bool"
        FROM authors a
        LEFT JOIN author_annotations aa ON a.id = aa.author
        WHERE a.id = ANY($1)
        "#,
        &author_ids
    )
    .fetch_all(&db.0)
    .await
    .unwrap();

    authors.sort_by(|a, b| {
        let a_pos = author_ids.iter().position(|i| *i == a.id).unwrap();
        let b_pos = author_ids.iter().position(|i| *i == b.id).unwrap();

        a_pos.cmp(&b_pos)
    });

    let page: Page<Author> = Page::new(authors, total.try_into().unwrap(), &pagination);

    Json(page)
}

pub async fn get_authors_router() -> Router {
    Router::new()
        .route("/", get(get_authors))
        .route("/random", get(get_random_author))
        .route("/:author_id", get(get_author))
        .route("/:author_id/annotation", get(get_author_annotation))
        .route("/:author_id/books", get(get_author_books))
        .route(
            "/:author_id/available_types",
            get(get_author_books_available_types),
        )
        .route("/search/:query", get(search_authors))
}
