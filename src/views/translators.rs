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
    meilisearch::{clamp_total_hits, AuthorMeili, MEILI_CLIENT, MEILI_TIMEOUT},
    serializers::{
        allowed_langs::AllowedLangs,
        author::Author,
        pagination::{Page, PageWithParent, Pagination},
        translator::TranslatorBook,
    },
};

use super::{common::query::normalize_query, Database};

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

    let books = sqlx::query_as::<_, TranslatorBook>(
        r#"
        SELECT
            b.id,
            b.title,
            b.lang,
            b.file_type,
            b.year,
            available_types(b.file_type) AS available_types,
            b.uploaded,
            COALESCE(
                (
                    SELECT
                        ARRAY_AGG(
                            ROW(
                                authors.id,
                                authors.first_name,
                                authors.last_name,
                                COALESCE(authors.middle_name, ''),
                                aa.author IS NOT NULL
                            )::author_type
                        )
                    FROM book_authors
                    JOIN authors ON authors.id = book_authors.author
                    LEFT JOIN author_annotations aa ON aa.author = authors.id
                    WHERE book_authors.book = b.id
                ),
                ARRAY[]::author_type[]
            ) AS authors,
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
            ) AS sequences,
            EXISTS(
                SELECT * FROM book_annotations WHERE book = b.id
            ) AS annotation_exists
        FROM books b
        JOIN translations t ON b.id = t.book
        WHERE
            b.is_deleted = false
            AND t.author = $1
            AND b.lang = ANY($2)
        ORDER BY t.position, b.title ASC, b.id ASC
        OFFSET $3
        LIMIT $4
        "#,
    )
    .bind(translator_id)
    .bind(&allowed_langs)
    .bind(pagination.offset())
    .bind(pagination.size)
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
        -- fb2 expansion is source-independent today because "flibusta" is the only source in this DB; add a source check here if a second source is ever introduced (see docs/specs/11-duplication-dead-code.md#11.2).
        SELECT DISTINCT unnest(
            available_types(b.file_type)
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
    let query = normalize_query(&query);
    let query = query.as_str();

    if query.is_empty() {
        let page: Page<Author> = Page::new(vec![], 0, &pagination);

        return Ok(Json(page));
    }

    let client = &MEILI_CLIENT;

    let authors_index = client.index("authors");

    let filter = format!("translator_langs IN [{}]", allowed_langs.join(", "));

    let result = tokio::time::timeout(
        *MEILI_TIMEOUT,
        authors_index
            .search()
            .with_query(query)
            .with_filter(&filter)
            .with_offset(pagination.offset_usize())
            .with_limit(pagination.limit_usize())
            .execute::<AuthorMeili>(),
    )
    .await
    .map_err(|_| ApiError::MeiliTimeout)??;

    let total = clamp_total_hits(result.estimated_total_hits);
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

    let page: Page<Author> = Page::new(translators, total, &pagination);

    Ok(Json(page))
}

pub fn get_translators_router() -> Router {
    Router::new()
        .route("/{translator_id}/books", get(get_translated_books))
        .route(
            "/{translator_id}/available_types",
            get(get_translated_books_available_types),
        )
        .route("/search/{query}", get(search_translators))
}
