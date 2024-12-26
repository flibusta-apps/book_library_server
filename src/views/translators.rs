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
        author::Author,
        book::BaseBook,
        pagination::{Page, PageWithParent, Pagination},
        sequence::Sequence,
        translator::TranslatorBook,
    },
};

use super::Database;

async fn get_translated_books(
    db: Database,
    Path(translator_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let translator = sqlx::query_as!(
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
        translator_id
    )
    .fetch_optional(&db.0)
    .await
    .unwrap();

    let translator = match translator {
        Some(translator) => translator,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let books_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM books b
        JOIN book_authors ba ON b.id = ba.book
        WHERE
            b.is_deleted = false
            AND ba.author = $1
            AND b.lang = ANY($2)
        "#,
        translator_id,
        &allowed_langs
    )
    .fetch_one(&db.0)
    .await
    .unwrap()
    .unwrap();

    let books = sqlx::query_as!(
        TranslatorBook,
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
                FROM book_authors
                JOIN authors ON authors.id = book_authors.author
                WHERE book_authors.book = b.id
            ) AS "authors!: Vec<Author>",
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
        "#,
    )
        .fetch_all(&db.0)
        .await
        .unwrap();

    let page: PageWithParent<TranslatorBook, Author> =
        PageWithParent::new(translator, books, books_count, &pagination);

    Json(page).into_response()
}

async fn get_translated_books_available_types(
    db: Database,
    Path(translator_id): Path<i32>,
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
        WHERE
            b.is_deleted = false
            AND ba.author = $1
            AND b.lang = ANY($2)
        "#,
        translator_id,
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

async fn search_translators(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let client = get_meili_client();

    let authors_index = client.index("authors");

    let filter = format!("translator_langs IN [{}]", allowed_langs.join(", "));

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
    let translator_ids: Vec<i32> = result.hits.iter().map(|a| a.result.id).collect();

    let mut translators = sqlx::query_as!(
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
        &translator_ids
    )
    .fetch_all(&db.0)
    .await
    .unwrap();

    translators.sort_by(|a, b| {
        let a_pos = translator_ids.iter().position(|i| *i == a.id).unwrap();
        let b_pos = translator_ids.iter().position(|i| *i == b.id).unwrap();

        a_pos.cmp(&b_pos)
    });

    let page: Page<Author> = Page::new(translators, total.try_into().unwrap(), &pagination);

    Json(page)
}

pub async fn get_translators_router() -> Router {
    Router::new()
        .route("/:translator_id/books", get(get_translated_books))
        .route(
            "/:translator_id/available_types",
            get(get_translated_books_available_types),
        )
        .route("/search/:query", get(search_translators))
}
