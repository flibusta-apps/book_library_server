//! Shared SQL building blocks for `books.rs` handlers.
//!
//! `query_detail_book!` consolidates the three previously-duplicated
//! `DetailBook` SELECT queries (`get_book`, `get_random_book`,
//! `get_remote_book`) into a single source of truth, using sqlx's
//! string-literal concatenation support in `query_as!`/`query_scalar!`
//! (`Punctuated::<LitStr, Token![+]>` parsing) to inject a caller-provided
//! `WHERE` clause while keeping full compile-time / `.sqlx`-offline
//! verification. See docs/specs/11-duplication-dead-code.md#11.1.

/// Builds a `DetailBook` query. `$where` is a string literal WHERE-clause
/// fragment (without the `WHERE` keyword itself); `$args` are the bind
/// parameters referenced by that fragment, in order.
macro_rules! query_detail_book {
    ($where:literal, $($args:tt)*) => {
        ::sqlx::query_as!(
            $crate::serializers::book::DetailBook,
            r#"
            SELECT
                b.id,
                b.title,
                b.lang,
                b.file_type,
                b.year,
                -- fb2 expansion is source-independent today because "flibusta" is the only source in this DB; add a source check here if a second source is ever introduced (see docs/specs/11-duplication-dead-code.md#11.2).
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
                                    COALESCE(authors.middle_name, ''),
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
                                    COALESCE(authors.middle_name, ''),
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
                COALESCE(
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
                    ),
                    ARRAY[]::genre_type[]
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
            WHERE "# + $where,
            $($args)*
        )
    };
}

pub(crate) use query_detail_book;

/// Shared `COUNT(*)` for the `BookFilter` predicate used by both `get_books`
/// and `get_base_books`.
pub(crate) async fn count_books(
    pool: &sqlx::PgPool,
    filter: &crate::serializers::book::BookFilter,
) -> Result<i64, crate::error::ApiError> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM books
        WHERE lang = ANY($1) AND
        ($2::boolean IS NULL OR is_deleted = $2) AND
        ($3::date IS NULL OR uploaded >= $3) AND
        ($4::date IS NULL OR uploaded <= $4) AND
        ($5::integer IS NULL OR id >= $5) AND
        ($6::integer IS NULL OR id <= $6)
        "#,
        &filter.allowed_langs,
        filter.is_deleted,
        filter.uploaded_gte,
        filter.uploaded_lte,
        filter.id_gte,
        filter.id_lte,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    Ok(count)
}
