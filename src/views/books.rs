use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::{
    error::ApiError,
    meilisearch::{BookMeili, MEILI_CLIENT, MEILI_TIMEOUT},
    serializers::{
        allowed_langs::AllowedLangs,
        author::Author,
        book::{BaseBook, Book, BookFilter, DetailBook, RandomBookFilter, RemoteBook},
        book_annotation::BookAnnotation,
        genre::Genre,
        pagination::{Page, Pagination},
        sequence::Sequence,
        source::Source,
    },
};

use super::{
    book_sql::{count_books, query_detail_book},
    common::get_random_item::get_random_item,
    Database,
};

pub async fn get_books(
    db: Database,
    axum_extra::extract::Query(book_filter): axum_extra::extract::Query<BookFilter>,
    pagination: Query<Pagination>,
) -> Result<impl IntoResponse, ApiError> {
    let books_count = count_books(&db.0, &book_filter).await?;

    let books = sqlx::query_as!(
        RemoteBook,
        r#"
        SELECT
            b.id,
            b.title,
            b.lang,
            b.file_type,
            b.year,
            available_types(b.file_type) AS "available_types!: Vec<String>",
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
            ) AS "authors!: Vec<Author>",
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
                    FROM translations
                    JOIN authors ON authors.id = translations.author
                    LEFT JOIN author_annotations aa ON aa.author = authors.id
                    WHERE translations.book = b.id
                ),
                ARRAY[]::author_type[]
            ) AS "translators!: Vec<Author>",
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
            ) AS "annotation_exists!: bool",
            (
                SELECT
                    ROW(
                        sources.id,
                        sources.name
                    )::source_type
                FROM sources
                WHERE sources.id = b.source
            ) AS "source!: Source",
            b.remote_id
        FROM books b
        WHERE lang = ANY($1) AND
        ($2::boolean IS NULL OR is_deleted = $2) AND
        ($3::date IS NULL OR uploaded >= $3) AND
        ($4::date IS NULL OR uploaded <= $4) AND
        ($5::integer IS NULL OR id >= $5) AND
        ($6::integer IS NULL OR id <= $6)
        ORDER BY b.id ASC
        OFFSET $7
        LIMIT $8
        "#,
        &book_filter.allowed_langs,
        book_filter.is_deleted,
        book_filter.uploaded_gte,
        book_filter.uploaded_lte,
        book_filter.id_gte,
        book_filter.id_lte,
        pagination.offset(),
        pagination.size,
    )
    .fetch_all(&db.0)
    .await?;

    let page: Page<RemoteBook> = Page::new(books, books_count, &pagination);

    Ok(Json(page))
}

pub async fn get_base_books(
    db: Database,
    axum_extra::extract::Query(book_filter): axum_extra::extract::Query<BookFilter>,
    pagination: Query<Pagination>,
) -> Result<impl IntoResponse, ApiError> {
    let books_count = count_books(&db.0, &book_filter).await?;

    let books = sqlx::query_as!(
        BaseBook,
        r#"
        SELECT
            b.id,
            available_types(b.file_type) AS "available_types!: Vec<String>"
        FROM books b
        WHERE lang = ANY($1) AND
        ($2::boolean IS NULL OR is_deleted = $2) AND
        ($3::date IS NULL OR uploaded >= $3) AND
        ($4::date IS NULL OR uploaded <= $4) AND
        ($5::integer IS NULL OR id >= $5) AND
        ($6::integer IS NULL OR id <= $6)
        ORDER BY b.id ASC
        OFFSET $7
        LIMIT $8
        "#,
        &book_filter.allowed_langs,
        book_filter.is_deleted,
        book_filter.uploaded_gte,
        book_filter.uploaded_lte,
        book_filter.id_gte,
        book_filter.id_lte,
        pagination.offset(),
        pagination.size,
    )
    .fetch_all(&db.0)
    .await?;

    let page: Page<BaseBook> = Page::new(books, books_count, &pagination);

    Ok(Json(page))
}

pub async fn get_random_book(
    db: Database,
    axum_extra::extract::Query(book_filter): axum_extra::extract::Query<RandomBookFilter>,
) -> Result<impl IntoResponse, ApiError> {
    let book_id = {
        let client = &MEILI_CLIENT;

        let books_index = client.index("books");

        let filter = {
            let langs_filter = format!("lang IN [{}]", book_filter.allowed_langs.join(", "));
            let genre_filter = match book_filter.genre {
                Some(v) => format!(" AND genres = {v}"),
                None => "".to_string(),
            };

            format!("{langs_filter}{genre_filter}")
        };

        get_random_item::<BookMeili>(books_index, filter).await?
    };

    let book = query_detail_book!("b.id = $1", book_id)
        .fetch_optional(&db.0)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json::<DetailBook>(book))
}

pub async fn get_remote_book(
    db: Database,
    Path((source_id, remote_id)): Path<(i16, i32)>,
) -> Result<impl IntoResponse, ApiError> {
    let book = query_detail_book!("b.source = $1 AND b.remote_id = $2", source_id, remote_id)
        .fetch_optional(&db.0)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json::<DetailBook>(book))
}

pub async fn search_books(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> Result<impl IntoResponse, ApiError> {
    let query = query.trim();

    if query.is_empty() {
        let page: Page<Book> = Page::new(vec![], 0, &pagination);

        return Ok(Json(page));
    }

    let truncated_query: String = if query.chars().count() > 256 {
        query.chars().take(256).collect()
    } else {
        query.to_string()
    };
    let query = truncated_query.as_str();

    let client = &MEILI_CLIENT;

    let book_index = client.index("books");

    let filter = format!("lang IN [{}]", allowed_langs.join(", "));

    let result = tokio::time::timeout(
        MEILI_TIMEOUT,
        book_index
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
            .execute::<BookMeili>(),
    )
    .await
    .map_err(|_| ApiError::MeiliTimeout)??;

    let total = result.estimated_total_hits.unwrap_or(0);
    let book_ids: Vec<i32> = result.hits.iter().map(|a| a.result.id).collect();

    let mut books = sqlx::query_as!(
        Book,
        r#"
        SELECT
            b.id,
            b.title,
            b.lang,
            b.file_type,
            b.year,
            available_types(b.file_type) AS "available_types!: Vec<String>",
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
            ) AS "authors!: Vec<Author>",
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
                    FROM translations
                    JOIN authors ON authors.id = translations.author
                    LEFT JOIN author_annotations aa ON aa.author = authors.id
                    WHERE translations.book = b.id
                ),
                ARRAY[]::author_type[]
            ) AS "translators!: Vec<Author>",
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
        WHERE b.id = ANY($1)
        "#,
        &book_ids
    )
    .fetch_all(&db.0)
    .await?;

    let rank_map: std::collections::HashMap<i32, usize> = book_ids
        .iter()
        .enumerate()
        .map(|(idx, id)| (*id, idx))
        .collect();

    books.sort_by_key(|book| rank_map.get(&book.id).copied().unwrap_or(usize::MAX));

    let page: Page<Book> = Page::new(books, total.try_into().unwrap_or(0i64), &pagination);

    Ok(Json(page))
}

pub async fn get_book(
    db: Database,
    Path(book_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let book = query_detail_book!("b.id = $1", book_id)
        .fetch_optional(&db.0)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json::<DetailBook>(book))
}

pub async fn get_book_annotation(
    db: Database,
    Path(book_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let book_annotation = sqlx::query_as!(
        BookAnnotation,
        r#"
        SELECT
            id,
            title,
            text,
            file
        FROM book_annotations
        WHERE book = $1
        "#,
        book_id
    )
    .fetch_optional(&db.0)
    .await?;

    Ok(match book_annotation {
        Some(book_annotation) => Json::<BookAnnotation>(book_annotation).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    })
}

pub fn get_books_router() -> Router {
    Router::new()
        .route("/", get(get_books))
        .route("/base", get(get_base_books))
        .route("/random", get(get_random_book))
        .route("/remote/{source_id}/{remote_id}", get(get_remote_book))
        .route("/search/{query}", get(search_books))
        .route("/{book_id}", get(get_book))
        .route("/{book_id}/annotation", get(get_book_annotation))
}
