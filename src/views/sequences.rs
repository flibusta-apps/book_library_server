use std::collections::HashMap;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::{
    error::ApiError,
    meilisearch::{SequenceMeili, MEILI_CLIENT},
    serializers::{
        allowed_langs::AllowedLangs,
        author::Author,
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
) -> Result<impl IntoResponse, ApiError> {
    let sequence_id = {
        let client = &MEILI_CLIENT;

        let authors_index = client.index("sequences");

        let filter = format!("langs IN [{}]", allowed_langs.join(", "));

        get_random_item::<SequenceMeili>(authors_index, filter).await?
    };

    let sequence = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, name FROM sequences WHERE id = $1
        "#,
        sequence_id
    )
    .fetch_optional(&db.0)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json::<Sequence>(sequence))
}

async fn search_sequence(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> Result<impl IntoResponse, ApiError> {
    let query: String = query.trim().chars().take(256).collect();

    if query.is_empty() {
        let page: Page<Sequence> = Page::new(vec![], 0, &pagination);
        return Ok(Json(page));
    }

    let client = &MEILI_CLIENT;

    let sequence_index = client.index("sequences");

    let filter = format!("langs IN [{}]", allowed_langs.join(", "));

    let result = sequence_index
        .search()
        .with_query(&query)
        .with_filter(&filter)
        // `Pagination` validation guarantees `page >= 1` and `size` within
        // [1, MAX_PAGE_SIZE], so these `i64 -> usize` conversions cannot fail.
        .with_offset(
            pagination
                .offset()
                .try_into()
                .expect("pagination values are validated to be non-negative"),
        )
        .with_limit(
            pagination
                .size
                .try_into()
                .expect("pagination size is validated to be non-negative"),
        )
        .execute::<SequenceMeili>()
        .await?;

    let total = result.estimated_total_hits.unwrap_or(0);
    let sequence_ids: Vec<i32> = result.hits.iter().map(|a| a.result.id).collect();

    let mut sequences = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, name FROM sequences WHERE id = ANY($1)
        "#,
        &sequence_ids
    )
    .fetch_all(&db.0)
    .await?;

    let rank: HashMap<i32, usize> = sequence_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i))
        .collect();

    sequences.sort_by_key(|a| rank.get(&a.id).copied().unwrap_or(usize::MAX));

    let page: Page<Sequence> = Page::new(sequences, total.try_into().unwrap_or(0i64), &pagination);

    Ok(Json(page))
}

async fn get_sequence(
    db: Database,
    Path(sequence_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let sequence = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, name FROM sequences WHERE id = $1
        "#,
        sequence_id
    )
    .fetch_optional(&db.0)
    .await?;

    Ok(match sequence {
        Some(sequence) => Json::<Sequence>(sequence).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    })
}

async fn get_sequence_available_types(
    db: Database,
    Path(sequence_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
) -> Result<impl IntoResponse, ApiError> {
    let file_types = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT unnest(
            CASE WHEN b.file_type = 'fb2' THEN ARRAY['fb2', 'epub', 'mobi', 'fb2zip']::text[] ELSE ARRAY[b.file_type]::text[] END
        ) AS "file_type!: String"
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
        .await?;

    Ok(Json::<Vec<String>>(file_types))
}

async fn get_sequence_books(
    db: Database,
    Path(sequence_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> Result<impl IntoResponse, ApiError> {
    let sequence = sqlx::query_as!(
        Sequence,
        r#"
        SELECT id, name FROM sequences WHERE id = $1
        "#,
        sequence_id
    )
    .fetch_optional(&db.0)
    .await?;

    let sequence = match sequence {
        Some(v) => v,
        None => return Ok(StatusCode::NOT_FOUND.into_response()),
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
    .await?
    .unwrap_or(0);

    let books = sqlx::query_as!(
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
            COALESCE(
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
                ),
                ARRAY[]::author_type[]
            ) AS "authors!: Vec<Author>",
            COALESCE(
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
                ),
                ARRAY[]::author_type[]
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
        .await?;

    let page: PageWithParent<SequenceBook, Sequence> =
        PageWithParent::new(sequence, books, books_count, &pagination);

    Ok(Json(page).into_response())
}

pub async fn get_sequences_router() -> Router {
    Router::new()
        .route("/random", get(get_random_sequence))
        .route("/search/{query}", get(search_sequence))
        .route("/{sequence_id}", get(get_sequence))
        .route(
            "/{sequence_id}/available_types",
            get(get_sequence_available_types),
        )
        .route("/{sequence_id}/books", get(get_sequence_books))
}
