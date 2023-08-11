use std::collections::HashSet;

use axum::{Router, routing::get, extract::{Path, Query}, response::IntoResponse, Json, http::StatusCode};

use crate::{serializers::{pagination::{Pagination, Page, PageWithParent}, author::Author, translator::TranslatorBook, allowed_langs::AllowedLangs}, meilisearch::{get_meili_client, AuthorMeili}, prisma::{author, book::{self}, translator, book_author, book_sequence}};

use super::Database;


async fn get_translated_books(
    db: Database,
    Path(translator_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>,
    pagination: Query<Pagination>
) -> impl IntoResponse {
    let translator = db
        .author()
        .find_unique(
            author::id::equals(translator_id)
        )
        .with(
            author::author_annotation::fetch()
        )
        .exec()
        .await
        .unwrap();

    let translator = match translator {
        Some(translator) => translator,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let books_count = db
        .book()
        .count(vec![
            book::is_deleted::equals(false),
            book::translations::some(vec![
                translator::author_id::equals(translator_id)
            ]),
            book::lang::in_vec(allowed_langs.clone())
        ])
        .exec()
        .await
        .unwrap();

    let books = db
        .book()
        .find_many(vec![
            book::is_deleted::equals(false),
            book::translations::some(vec![
                translator::author_id::equals(translator_id)
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
            book::book_authors::fetch(vec![])
                .with(
                    book_author::author::fetch()
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

    let page: PageWithParent<TranslatorBook, Author> = PageWithParent::new(
        translator.into(),
        books.iter().map(|item| item.clone().into()).collect(),
        books_count,
        &pagination
    );

    Json(page).into_response()
}


async fn get_translated_books_available_types(
    db: Database,
    Path(translator_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>
) -> impl IntoResponse {
    let books = db
        .book()
        .find_many(vec![
            book::is_deleted::equals(false),
            book::translations::some(vec![
                translator::author_id::equals(translator_id)
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


async fn search_translators(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>,
    pagination: Query<Pagination>
) -> impl IntoResponse {
    let client = get_meili_client();

    let authors_index = client.index("authors");

    let filter = format!(
        "translator_langs IN [{}]",
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
    let translator_ids: Vec<i32> = result.hits.iter().map(|a| a.result.id).collect();

    let mut translators = db
        .author()
        .find_many(vec![
            author::id::in_vec(translator_ids.clone())
        ])
        .with(
            author::author_annotation::fetch()
        )
        .order_by(author::id::order(prisma_client_rust::Direction::Asc))
        .exec()
        .await
        .unwrap();

    translators.sort_by(|a, b| {
        let a_pos = translator_ids.iter().position(|i| *i == a.id).unwrap();
        let b_pos = translator_ids.iter().position(|i| *i == b.id).unwrap();

        a_pos.cmp(&b_pos)
    });

    let page: Page<Author> = Page::new(
        translators.iter().map(|item| item.clone().into()).collect(),
        total.try_into().unwrap(),
        &pagination
    );

    Json(page)
}


pub async fn get_translators_router() -> Router {
    Router::new()
        .route("/:translator_id/books", get(get_translated_books))
        .route("/:translator_id/available_types", get(get_translated_books_available_types))
        .route("/search/:query", get(search_translators))
}
