use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::{
    meilisearch::{get_meili_client, BookMeili},
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

use super::{common::get_random_item::get_random_item, Database};

pub async fn get_books(
    db: Database,
    axum_extra::extract::Query(book_filter): axum_extra::extract::Query<BookFilter>,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let books_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM books
        WHERE lang = ANY($1) AND
        ($2::boolean IS NULL OR is_deleted = $2) AND
        ($3::date IS NULL OR uploaded >= $3) AND
        ($4::date IS NULL OR uploaded <= $4) AND
        ($5::integer IS NULL OR id >= $5) AND
        ($6::integer IS NULL OR id <= $6)
        "#,
        &book_filter.allowed_langs,
        book_filter.is_deleted,
        book_filter.uploaded_gte,
        book_filter.uploaded_lte,
        book_filter.id_gte,
        book_filter.id_lte,
    )
    .fetch_one(&db.0)
    .await
    .unwrap()
    .unwrap();

    let books = sqlx::query_as!(
        RemoteBook,
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
        (pagination.page - 1) * pagination.size,
        pagination.size,
    )
        .fetch_all(&db.0)
        .await
        .unwrap();

    let page: Page<RemoteBook> = Page::new(books, books_count, &pagination);

    Json(page)
}

pub async fn get_base_books(
    db: Database,
    axum_extra::extract::Query(book_filter): axum_extra::extract::Query<BookFilter>,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let books_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM books
        WHERE lang = ANY($1) AND
        ($2::boolean IS NULL OR is_deleted = $2) AND
        ($3::date IS NULL OR uploaded >= $3) AND
        ($4::date IS NULL OR uploaded <= $4) AND
        ($5::integer IS NULL OR id >= $5) AND
        ($6::integer IS NULL OR id <= $6)
        "#,
        &book_filter.allowed_langs,
        book_filter.is_deleted,
        book_filter.uploaded_gte,
        book_filter.uploaded_lte,
        book_filter.id_gte,
        book_filter.id_lte,
    )
    .fetch_one(&db.0)
    .await
    .unwrap()
    .unwrap();

    let books = sqlx::query_as!(
        BaseBook,
        r#"
        SELECT
            b.id,
            CASE WHEN b.file_type = 'fb2' THEN ARRAY['fb2', 'epub', 'mobi', 'fb2zip']::text[] ELSE ARRAY[b.file_type]::text[] END AS "available_types!: Vec<String>"
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
        (pagination.page - 1) * pagination.size,
        pagination.size,
    )
        .fetch_all(&db.0)
        .await
        .unwrap();

    let page: Page<BaseBook> = Page::new(books, books_count, &pagination);

    Json(page)
}

pub async fn get_random_book(
    db: Database,
    axum_extra::extract::Query(book_filter): axum_extra::extract::Query<RandomBookFilter>,
) -> impl IntoResponse {
    let book_id = {
        let client = get_meili_client();

        let authors_index = client.index("books");

        let filter = {
            let langs_filter = format!("lang IN [{}]", book_filter.allowed_langs.join(", "));
            let genre_filter = match book_filter.genre {
                Some(v) => format!(" AND genres = {v}"),
                None => "".to_string(),
            };

            format!("{langs_filter}{genre_filter}")
        };

        get_random_item::<BookMeili>(authors_index, filter).await
    };

    let book = sqlx::query_as!(
        DetailBook,
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
            (
                SELECT
                    ARRAY_AGG(
                        ROW(
                            genres.id,
                            ROW(
                                sources.id,
                                sources.name
                            )::source_type,
                            genres.remote_id,
                            genres.code,
                            genres.description,
                            genres.meta
                        )::genre_type
                    )
                FROM book_genres
                JOIN genres ON genres.id = book_genres.genre
                JOIN sources ON sources.id = genres.source
                WHERE book_genres.book = b.id
            ) AS "genres!: Vec<Genre>",
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
            b.remote_id,
            b.is_deleted,
            b.pages
        FROM books b
        WHERE b.id = $1
        "#,
        book_id
    )
        .fetch_optional(&db.0)
        .await
        .unwrap()
        .unwrap();

    Json::<DetailBook>(book).into_response()
}

pub async fn get_remote_book(
    db: Database,
    Path((source_id, remote_id)): Path<(i16, i32)>,
) -> impl IntoResponse {
    let book = sqlx::query_as!(
        DetailBook,
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
            (
                SELECT
                    ARRAY_AGG(
                        ROW(
                            genres.id,
                            ROW(
                                sources.id,
                                sources.name
                            )::source_type,
                            remote_id,
                            genres.code,
                            genres.description,
                            genres.meta
                        )::genre_type
                    )
                FROM book_genres
                JOIN genres ON genres.id = book_genres.genre
                JOIN sources ON sources.id = genres.source
                WHERE book_genres.book = b.id
            ) AS "genres!: Vec<Genre>",
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
            b.remote_id,
            b.is_deleted,
            b.pages
        FROM books b
        WHERE b.source = $1 AND b.remote_id = $2
        "#,
        source_id,
        remote_id
    )
        .fetch_optional(&db.0)
        .await
        .unwrap();

    match book {
        Some(book) => Json::<DetailBook>(book).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn search_books(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let client = get_meili_client();

    let book_index = client.index("books");

    let filter = format!("lang IN [{}]", allowed_langs.join(", "));

    let result = book_index
        .search()
        .with_query(&query)
        .with_filter(&filter)
        .with_offset(
            ((pagination.page - 1) * pagination.size)
                .try_into()
                .unwrap(),
        )
        .with_limit(pagination.size.try_into().unwrap())
        .execute::<BookMeili>()
        .await
        .unwrap();

    let total = result.estimated_total_hits.unwrap();
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
        WHERE b.id = ANY($1)
        "#,
        &book_ids
    )
        .fetch_all(&db.0)
        .await
        .unwrap();

    books.sort_by(|a, b| {
        let a_pos = book_ids.iter().position(|i| *i == a.id).unwrap();
        let b_pos = book_ids.iter().position(|i| *i == b.id).unwrap();

        a_pos.cmp(&b_pos)
    });

    let page: Page<Book> = Page::new(books, total.try_into().unwrap(), &pagination);

    Json(page)
}

pub async fn get_book(db: Database, Path(book_id): Path<i32>) -> impl IntoResponse {
    let book = sqlx::query_as!(
        DetailBook,
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
            (
                SELECT
                    ARRAY_AGG(
                        ROW(
                            genres.id,
                            ROW(
                                sources.id,
                                sources.name
                            )::source_type,
                            genres.remote_id,
                            genres.code,
                            genres.description,
                            genres.meta
                        )::genre_type
                    )
                FROM book_genres
                JOIN genres ON genres.id = book_genres.genre
                JOIN sources ON sources.id = genres.source
                WHERE book_genres.book = b.id
            ) AS "genres!: Vec<Genre>",
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
            b.remote_id,
            b.is_deleted,
            b.pages
        FROM books b
        WHERE b.id = $1
        "#,
        book_id
    )
        .fetch_optional(&db.0)
        .await
        .unwrap();

    match book {
        Some(book) => Json::<DetailBook>(book).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_book_annotation(db: Database, Path(book_id): Path<i32>) -> impl IntoResponse {
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
    .await
    .unwrap();

    match book_annotation {
        Some(book_annotation) => Json::<BookAnnotation>(book_annotation).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_books_router() -> Router {
    Router::new()
        .route("/", get(get_books))
        .route("/base/", get(get_base_books))
        .route("/random", get(get_random_book))
        .route("/remote/:source_id/:remote_id", get(get_remote_book))
        .route("/search/:query", get(search_books))
        .route("/:book_id", get(get_book))
        .route("/:book_id/annotation", get(get_book_annotation))
}
