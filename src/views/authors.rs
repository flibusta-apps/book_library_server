use std::collections::HashSet;

use axum::{Router, extract::{Query, Path}, Json, response::IntoResponse, routing::get, http::StatusCode};

use rand::Rng;

use crate::{prisma::{author, author_annotation::{self}, book, book_author, translator, book_sequence}, serializers::{pagination::{Pagination, Page, PageWithParent}, author::{Author, AuthorBook}, author_annotation::AuthorAnnotation, allowed_langs::AllowedLangs}, meilisearch::{get_meili_client, AuthorMeili}};

use super::Database;


async fn get_authors(
    db: Database,
    pagination: Query<Pagination>
) -> impl IntoResponse {
    let authors_count = db
        .author()
        .count(vec![])
        .exec()
        .await
        .unwrap();

    let authors = db
        .author()
        .find_many(vec![])
        .with(
            author::author_annotation::fetch()
        )
        .order_by(author::id::order(prisma_client_rust::Direction::Asc))
        .skip((pagination.page - 1) * pagination.size)
        .take(pagination.size)
        .exec()
        .await
        .unwrap();

    let page: Page<Author> = Page::new(
        authors.iter().map(|item| item.clone().into()).collect(),
        authors_count,
        &pagination
    );

    Json(page)
}


async fn get_random_author(
    db: Database,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>
) -> impl IntoResponse {
    let client = get_meili_client();

    let authors_index = client.index("authors");

    let filter = format!(
        "author_langs IN [{}]",
        allowed_langs.join(", ")
    );

    let result = authors_index
        .search()
        .with_filter(&filter)
        .execute::<AuthorMeili>()
        .await
        .unwrap();

    let author_id = {
        let offset: usize = rand::thread_rng().gen_range(0..result.estimated_total_hits.unwrap().try_into().unwrap());

        let result = authors_index
            .search()
            .with_limit(1)
            .with_offset(offset)
            .execute::<AuthorMeili>()
            .await
            .unwrap();

        let author = &result.hits.get(0).unwrap().result;

        author.id
    };

    let author = db
        .author()
        .find_unique(
            author::id::equals(author_id)
        )
        .with(
            author::author_annotation::fetch()
        )
        .exec()
        .await
        .unwrap()
        .unwrap();

    Json::<Author>(author.into())
}


async fn get_author(
    db: Database,
    Path(author_id): Path<i32>
) -> impl IntoResponse {
    let author = db
        .author()
        .find_unique(
            author::id::equals(author_id)
        )
        .with(
            author::author_annotation::fetch()
        )
        .exec()
        .await
        .unwrap();

    match author {
        Some(author) => Json::<Author>(author.into()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}


async fn get_author_annotation(
    db: Database,
    Path(author_id): Path<i32>,
) -> impl IntoResponse {
    let author_annotation = db
        .author_annotation()
        .find_unique(
            author_annotation::author_id::equals(author_id)
        )
        .exec()
        .await
        .unwrap();

    match author_annotation {
        Some(annotation) => Json::<AuthorAnnotation>(annotation.into()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}


async fn get_author_books(
    db: Database,
    Path(author_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>,
    pagination: Query<Pagination>
) -> impl IntoResponse {
    let author = db
        .author()
        .find_unique(
            author::id::equals(author_id)
        )
        .with(
            author::author_annotation::fetch()
        )
        .exec()
        .await
        .unwrap();

    let author = match author {
        Some(author) => author,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let books_count = db
        .book()
        .count(vec![
            book::book_authors::some(vec![
                book_author::author_id::equals(author_id)
            ]),
            book::lang::in_vec(allowed_langs.clone())
        ])
        .exec()
        .await
        .unwrap();

    let books = db
        .book()
        .find_many(vec![
            book::book_authors::some(vec![
                book_author::author_id::equals(author_id)
            ]),
            book::lang::in_vec(allowed_langs)
        ])
        .with(
            book::source::fetch()
        )
        .with(
            book::book_annotation::fetch()
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
        .order_by(book::id::order(prisma_client_rust::Direction::Asc))
        .skip((pagination.page - 1) * pagination.size)
        .take(pagination.size)
        .exec()
        .await
        .unwrap();

    let page: PageWithParent<AuthorBook, Author> = PageWithParent::new(
        author.into(),
        books.iter().map(|item| item.clone().into()).collect(),
        books_count,
        &pagination
    );

    Json(page).into_response()
}


async fn get_author_books_available_types(
    db: Database,
    Path(author_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>
) -> impl IntoResponse {
    let books = db
        .book()
        .find_many(vec![
            book::book_authors::some(vec![
                book_author::author_id::equals(author_id)
            ]),
            book::lang::in_vec(allowed_langs)
        ])
        .exec()
        .await
        .unwrap();

    let mut file_types: HashSet<String> = HashSet::new();

    for book in books {
        file_types.insert(book.file_type.clone());
    }

    if file_types.contains(&"fb2".to_string()) {
        file_types.insert("epub".to_string());
        file_types.insert("mobi".to_string());
        file_types.insert("fb2zip".to_string());
    }

    Json::<Vec<String>>(file_types.into_iter().collect())
}


async fn search_authors(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>,
    pagination: Query<Pagination>
) -> impl IntoResponse {
    let client = get_meili_client();

    let authors_index = client.index("authors");

    let filter = format!(
        "author_langs IN [{}]",
        allowed_langs.join(", ")
    );

    let result = authors_index
        .search()
        .with_query(&query)
        .with_filter(&filter)
        .with_offset(((pagination.page - 1) * pagination.size).try_into().unwrap())
        .with_limit(pagination.size.try_into().unwrap())
        .execute::<AuthorMeili>()
        .await
        .unwrap();

    let total = result.estimated_total_hits.unwrap();
    let author_ids: Vec<i32> = result.hits.iter().map(|a| a.result.id).collect();

    let mut authors = db
        .author()
        .find_many(vec![
            author::id::in_vec(author_ids.clone())
        ])
        .with(
            author::author_annotation::fetch()
        )
        .order_by(author::id::order(prisma_client_rust::Direction::Asc))
        .exec()
        .await
        .unwrap();

    authors.sort_by(|a, b| {
        let a_pos = author_ids.iter().position(|i| *i == a.id).unwrap();
        let b_pos = author_ids.iter().position(|i| *i == b.id).unwrap();

        a_pos.cmp(&b_pos)
    });

    let page: Page<Author> = Page::new(
        authors.iter().map(|item| item.clone().into()).collect(),
        total.try_into().unwrap(),
        &pagination
    );

    Json(page)
}


pub async fn get_authors_router() -> Router {
    Router::new()
        .route("/", get(get_authors))
        .route("/random", get(get_random_author))
        .route("/:author_id", get(get_author))
        .route("/:author_id/annotation", get(get_author_annotation))
        .route("/:author_id/books", get(get_author_books))
        .route("/:author_id/available_types", get(get_author_books_available_types))
        .route("/search/:query", get(search_authors))
}
