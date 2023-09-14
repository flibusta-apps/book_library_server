use axum::{Router, routing::get, extract::{Query, Path}, Json, response::IntoResponse, http::StatusCode};
use prisma_client_rust::Direction;

use crate::{serializers::{book::{BookFilter, RemoteBook, BaseBook, DetailBook, RandomBookFilter, Book}, pagination::{Pagination, Page}, book_annotation::BookAnnotation, allowed_langs::AllowedLangs}, prisma::{book::{self}, book_author, author, translator, book_sequence, book_genre, book_annotation, genre}, meilisearch::{get_meili_client, BookMeili}};

use super::{Database, common::get_random_item::get_random_item};


pub async fn get_books(
    db: Database,
    axum_extra::extract::Query(book_filter): axum_extra::extract::Query<BookFilter>,
    pagination: Query<Pagination>
) -> impl IntoResponse {
    let filter = book_filter.get_filter_vec();

    let books_count = db
        .book()
        .count(filter.clone())
        .exec()
        .await
        .unwrap();

    let books = db
        .book()
        .find_many(filter)
        .with(
            book::book_annotation::fetch()
        )
        .with(
            book::source::fetch()
        )
        .with(
            book::book_authors::fetch(vec![])
                .with(
                    book_author::author::fetch()
                        .with(
                            author::author_annotation::fetch()
                        )
                )
        )
        .with(
            book::translations::fetch(vec![])
                .with(
                    translator::author::fetch()
                        .with(
                            author::author_annotation::fetch()
                        )
                )
        )
        .with(
            book::book_sequences::fetch(vec![])
                .with(
                    book_sequence::sequence::fetch()
                )
        )
        .order_by(book::id::order(Direction::Asc))
        .skip((pagination.page - 1) * pagination.size)
        .take(pagination.size)
        .exec()
        .await
        .unwrap();

    let page: Page<RemoteBook> = Page::new(
        books.iter().map(|item| item.clone().into()).collect(),
        books_count,
        &pagination
    );

    Json(page)
}

pub async fn get_base_books(
    db: Database,
    axum_extra::extract::Query(book_filter): axum_extra::extract::Query<BookFilter>,
    pagination: Query<Pagination>
) -> impl IntoResponse {
    let filter = book_filter.get_filter_vec();

    let books_count = db
        .book()
        .count(filter.clone())
        .exec()
        .await
        .unwrap();

    let books = db
        .book()
        .find_many(filter)
        .with(
            book::source::fetch()
        )
        .order_by(book::id::order(Direction::Asc))
        .skip((pagination.page - 1) * pagination.size)
        .take(pagination.size)
        .exec()
        .await
        .unwrap();

    let page: Page<BaseBook> = Page::new(
        books.iter().map(|item| item.clone().into()).collect(),
        books_count,
        &pagination
    );

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
            let langs_filter = format!(
                "lang IN [{}]",
                book_filter.allowed_langs.join(", ")
            );
            let genre_filter = match book_filter.genre {
                Some(v) => format!(" AND genres = {v}"),
                None => "".to_string(),
            };

            format!("{langs_filter}{genre_filter}")
        };

        get_random_item::<BookMeili>(
            authors_index,
            filter
        ).await
    };

    let book = db
        .book()
        .find_unique(book::id::equals(book_id))
        .with(
            book::book_annotation::fetch()
        )
        .with(
            book::source::fetch()
        )
        .with(
            book::book_authors::fetch(vec![])
                .with(
                    book_author::author::fetch()
                        .with(
                            author::author_annotation::fetch()
                        )
                )
        )
        .with(
            book::translations::fetch(vec![])
                .with(
                    translator::author::fetch()
                        .with(
                            author::author_annotation::fetch()
                        )
                )
        )
        .with(
            book::book_sequences::fetch(vec![])
                .with(
                    book_sequence::sequence::fetch()
                )
        )
        .with(
            book::book_genres::fetch(vec![])
                .with(
                    book_genre::genre::fetch()
                        .with(
                            genre::source::fetch()
                        )
                )
        )
        .exec()
        .await
        .unwrap()
        .unwrap();

    Json::<DetailBook>(book.into()).into_response()
}

pub async fn get_remote_book(
    db: Database,
    Path((source_id, remote_id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let book = db
        .book()
        .find_unique(book::source_id_remote_id(source_id, remote_id))
        .with(
            book::book_annotation::fetch()
        )
        .with(
            book::source::fetch()
        )
        .with(
            book::book_authors::fetch(vec![])
                .with(
                    book_author::author::fetch()
                        .with(
                            author::author_annotation::fetch()
                        )
                )
        )
        .with(
            book::translations::fetch(vec![])
                .with(
                    translator::author::fetch()
                        .with(
                            author::author_annotation::fetch()
                        )
                )
        )
        .with(
            book::book_sequences::fetch(vec![])
                .with(
                    book_sequence::sequence::fetch()
                )
        )
        .with(
            book::book_genres::fetch(vec![])
                .with(
                    book_genre::genre::fetch()
                        .with(
                            genre::source::fetch()
                        )
                )
        )
        .exec()
        .await
        .unwrap();

    match book {
        Some(book) => Json::<DetailBook>(book.into()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn search_books(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>,
    pagination: Query<Pagination>
) -> impl IntoResponse {
    let client = get_meili_client();

    let book_index = client.index("books");

    let filter = format!(
        "lang IN [{}]",
        allowed_langs.join(", ")
    );

    let result = book_index
        .search()
        .with_query(&query)
        .with_filter(&filter)
        .with_offset(((pagination.page - 1) * pagination.size).try_into().unwrap())
        .with_limit(pagination.size.try_into().unwrap())
        .execute::<BookMeili>()
        .await
        .unwrap();

    let total = result.estimated_total_hits.unwrap();
    let book_ids: Vec<i32> = result.hits.iter().map(|a| a.result.id).collect();

    let mut books = db
        .book()
        .find_many(vec![book::id::in_vec(book_ids.clone())])
        .with(
            book::book_annotation::fetch()
        )
        .with(
            book::source::fetch()
        )
        .with(
            book::book_authors::fetch(vec![])
                .with(
                    book_author::author::fetch()
                        .with(
                            author::author_annotation::fetch()
                        )
                )
        )
        .with(
            book::translations::fetch(vec![])
                .with(
                    translator::author::fetch()
                        .with(
                            author::author_annotation::fetch()
                        )
                )
        )
        .with(
            book::book_sequences::fetch(vec![])
                .with(
                    book_sequence::sequence::fetch()
                )
        )
        .exec()
        .await
        .unwrap();

    books.sort_by(|a, b| {
        let a_pos = book_ids.iter().position(|i| *i == a.id).unwrap();
        let b_pos = book_ids.iter().position(|i| *i == b.id).unwrap();

        a_pos.cmp(&b_pos)
    });

    let page: Page<Book> = Page::new(
        books.iter().map(|item| item.clone().into()).collect(),
        total.try_into().unwrap(),
        &pagination
    );

    Json(page)
}

pub async fn get_book(
    db: Database,
    Path(book_id): Path<i32>,
) -> impl IntoResponse {
    let book = db
        .book()
        .find_unique(book::id::equals(book_id))
        .with(
            book::book_annotation::fetch()
        )
        .with(
            book::source::fetch()
        )
        .with(
            book::book_authors::fetch(vec![])
                .with(
                    book_author::author::fetch()
                        .with(
                            author::author_annotation::fetch()
                        )
                )
        )
        .with(
            book::translations::fetch(vec![])
                .with(
                    translator::author::fetch()
                        .with(
                            author::author_annotation::fetch()
                        )
                )
        )
        .with(
            book::book_sequences::fetch(vec![])
                .with(
                    book_sequence::sequence::fetch()
                )
        )
        .with(
            book::book_genres::fetch(vec![])
                .with(
                    book_genre::genre::fetch()
                        .with(
                            genre::source::fetch()
                        )
                )
        )
        .exec()
        .await
        .unwrap();

    match book {
        Some(book) => Json::<DetailBook>(book.into()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_book_annotation(
    db: Database,
    Path(book_id): Path<i32>,
) -> impl IntoResponse {
    let book_annotation = db
        .book_annotation()
        .find_unique(
            book_annotation::book_id::equals(book_id)
        )
        .exec()
        .await
        .unwrap();

    match book_annotation {
        Some(book_annotation) => Json::<BookAnnotation>(book_annotation.into()).into_response(),
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
