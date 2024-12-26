use std::collections::HashSet;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::{
    meilisearch::{get_meili_client, SequenceMeili},
    serializers::{
        allowed_langs::AllowedLangs,
        author::Author,
        book::BaseBook,
        pagination::{Page, PageWithParent, Pagination},
        sequence::{Sequence, SequenceBook},
    },
};

use super::{common::get_random_item::get_random_item, Database};

async fn get_random_sequence(
    db: Database,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
) -> impl IntoResponse {
    let sequence_id = {
        let client = get_meili_client();

        let authors_index = client.index("sequences");

        let filter = format!("langs IN [{}]", allowed_langs.join(", "));

        get_random_item::<SequenceMeili>(authors_index, filter).await
    };

    let sequence = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, name FROM sequences WHERE id = $1
        "#,
        sequence_id
    )
    .fetch_one(&db.0)
    .await
    .unwrap();

    Json::<Sequence>(sequence)
}

async fn search_sequence(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let client = get_meili_client();

    let sequence_index = client.index("sequences");

    let filter = format!("langs IN [{}]", allowed_langs.join(", "));

    let result = sequence_index
        .search()
        .with_query(&query)
        .with_filter(&filter)
        .with_offset(
            ((pagination.page - 1) * pagination.size)
                .try_into()
                .unwrap(),
        )
        .with_limit(pagination.size.try_into().unwrap())
        .execute::<SequenceMeili>()
        .await
        .unwrap();

    let total = result.estimated_total_hits.unwrap();
    let sequence_ids: Vec<i32> = result.hits.iter().map(|a| a.result.id).collect();

    let mut sequences = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, name FROM sequences WHERE id = ANY($1)
        "#,
        &sequence_ids
    )
    .fetch_all(&db.0)
    .await
    .unwrap();

    sequences.sort_by(|a, b| {
        let a_pos = sequence_ids.iter().position(|i| *i == a.id).unwrap();
        let b_pos: usize = sequence_ids.iter().position(|i| *i == b.id).unwrap();

        a_pos.cmp(&b_pos)
    });

    let page: Page<Sequence> = Page::new(sequences, total.try_into().unwrap(), &pagination);

    Json(page)
}

async fn get_sequence(db: Database, Path(sequence_id): Path<i32>) -> impl IntoResponse {
    let sequence = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, name FROM sequences WHERE id = $1
        "#,
        sequence_id
    )
    .fetch_optional(&db.0)
    .await
    .unwrap();

    match sequence {
        Some(sequence) => Json::<Sequence>(sequence).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_sequence_available_types(
    db: Database,
    Path(sequence_id): Path<i32>,
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
        JOIN book_sequences bs ON b.id = bs.book
        WHERE
            b.is_deleted = FALSE AND
            bs.sequence = $1 AND
            b.lang = ANY($2)
        "#,
        sequence_id,
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

async fn get_sequence_books(
    db: Database,
    Path(sequence_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let sequence = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, name FROM sequences WHERE id = $1
        "#,
        sequence_id
    )
    .fetch_optional(&db.0)
    .await
    .unwrap();

    let sequence = match sequence {
        Some(v) => v,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let books_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM book_sequences bs
        JOIN books b ON b.id = bs.book
        WHERE
            b.is_deleted = FALSE AND
            bs.sequence = $1 AND
            b.lang = ANY($2)",
        sequence.id,
        &allowed_langs
    )
    .fetch_one(&db.0)
    .await
    .unwrap()
    .unwrap();

    let mut books = sqlx::query_as!(
        SequenceBook,
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
            EXISTS(
                SELECT * FROM book_annotations WHERE book = b.id
            ) AS "annotation_exists!: bool",
            bs.position
        FROM books b
        JOIN book_sequences bs ON b.id = bs.book
        WHERE
            b.is_deleted = FALSE AND
            bs.sequence = $1 AND
            b.lang = ANY($2)
        ORDER BY bs.position
        LIMIT $3 OFFSET $4
        "#,
        sequence.id,
        &allowed_langs,
        pagination.size,
        (pagination.page - 1) * pagination.size,
    )
        .fetch_all(&db.0)
        .await
        .unwrap();

    books.sort_by(|a, b| a.position.cmp(&b.position));

    let page: PageWithParent<SequenceBook, Sequence> =
        PageWithParent::new(sequence, books, books_count, &pagination);

    Json(page).into_response()
}

pub async fn get_sequences_router() -> Router {
    Router::new()
        .route("/random", get(get_random_sequence))
        .route("/search/:query", get(search_sequence))
        .route("/:sequence_id", get(get_sequence))
        .route(
            "/:sequence_id/available_types",
            get(get_sequence_available_types),
        )
        .route("/:sequence_id/books", get(get_sequence_books))
}
