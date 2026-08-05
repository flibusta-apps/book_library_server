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
    meilisearch::{AuthorMeili, MEILI_CLIENT},
    serializers::{
        allowed_langs::AllowedLangs,
        author::Author,
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
) -> Result<impl IntoResponse, ApiError> {
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
    .await?;

    let translator = match translator {
        Some(translator) => translator,
        None => return Ok(StatusCode::NOT_FOUND.into_response()),
    };

    let books_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM books b
        JOIN translations t ON b.id = t.book
        WHERE
            b.is_deleted = false
            AND t.author = $1
            AND b.lang = ANY($2)
        "#,
        translator_id,
        &allowed_langs
    )
    .fetch_one(&db.0)
    .await?
    .unwrap_or(0);

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
                                sequences.id,
                                sequences.name
                            )::sequence_type
                        )
                    FROM book_sequences
                    JOIN sequences ON sequences.id = book_sequences.sequence
                    WHERE book_sequences.book = b.id
                ),
                ARRAY[]::sequence_type[]
            ) AS "sequences!: Vec<Sequence>",
            EXISTS(
                SELECT * FROM book_annotations WHERE book = b.id
            ) AS "annotation_exists!: bool"
        FROM books b
        JOIN translations t ON b.id = t.book
        WHERE
            b.is_deleted = false
            AND t.author = $1
            AND b.lang = ANY($2)
        ORDER BY t.position, b.title ASC
        OFFSET $3
        LIMIT $4
        "#,
        translator_id,
        &allowed_langs,
        (pagination.page - 1) * pagination.size,
        pagination.size
    )
        .fetch_all(&db.0)
        .await?;

    let page: PageWithParent<TranslatorBook, Author> =
        PageWithParent::new(translator, books, books_count, &pagination);

    Ok(Json(page).into_response())
}

async fn get_translated_books_available_types(
    db: Database,
    Path(translator_id): Path<i32>,
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
        JOIN translations t ON b.id = t.book
        WHERE
            b.is_deleted = false
            AND t.author = $1
            AND b.lang = ANY($2)
        "#,
        translator_id,
        &allowed_langs
    )
        .fetch_all(&db.0)
        .await?;

    Ok(Json::<Vec<String>>(file_types))
}

async fn search_translators(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> Result<impl IntoResponse, ApiError> {
    let query = query.trim();

    if query.is_empty() {
        let page: Page<Author> = Page::new(vec![], 0, &pagination);

        return Ok(Json(page));
    }

    let truncated_query: String = if query.chars().count() > 256 {
        query.chars().take(256).collect()
    } else {
        query.to_string()
    };
    let query = truncated_query.as_str();

    let client = &MEILI_CLIENT;

    let authors_index = client.index("authors");

    let filter = format!("translator_langs IN [{}]", allowed_langs.join(", "));

    let result = authors_index
        .search()
        .with_query(query)
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
        .execute::<AuthorMeili>()
        .await?;

    let total = result.estimated_total_hits.unwrap_or(0);
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
    .await?;

    let rank_map: HashMap<i32, usize> = translator_ids
        .iter()
        .enumerate()
        .map(|(idx, id)| (*id, idx))
        .collect();

    translators
        .sort_by_key(|translator| rank_map.get(&translator.id).copied().unwrap_or(usize::MAX));

    let page: Page<Author> = Page::new(translators, total.try_into().unwrap_or(0i64), &pagination);

    Ok(Json(page))
}

pub async fn get_translators_router() -> Router {
    Router::new()
        .route("/{translator_id}/books", get(get_translated_books))
        .route(
            "/{translator_id}/available_types",
            get(get_translated_books_available_types),
        )
        .route("/search/{query}", get(search_translators))
}
