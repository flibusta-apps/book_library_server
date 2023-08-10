use std::collections::HashSet;

use axum::{Router, routing::get, extract::{Path, Query}, http::StatusCode, response::IntoResponse, Json};
use rand::Rng;

use crate::{prisma::{sequence, book_sequence, book, book_author, author, translator}, serializers::{sequence::{Sequence, SequenceBook}, allowed_langs::AllowedLangs, pagination::{PageWithParent, Pagination, Page}}, meilisearch::{get_meili_client, SequenceMeili}};

use super::Database;


async fn get_random_sequence(
    db: Database,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>
) -> impl IntoResponse {
    let client = get_meili_client();

    let authors_index = client.index("sequences");

    let filter = format!(
        "langs IN [{}]",
        allowed_langs.join(", ")
    );

    let result = authors_index
        .search()
        .with_filter(&filter)
        .execute::<SequenceMeili>()
        .await
        .unwrap();

    let sequence_id = {
        let offset: usize = rand::thread_rng().gen_range(0..result.estimated_total_hits.unwrap().try_into().unwrap());

        let result = authors_index
            .search()
            .with_limit(1)
            .with_offset(offset)
            .execute::<SequenceMeili>()
            .await
            .unwrap();

        let sequence = &result.hits.get(0).unwrap().result;

        sequence.id
    };

    let sequence = db
        .sequence()
        .find_unique(
            sequence::id::equals(sequence_id)
        )
        .exec()
        .await
        .unwrap()
        .unwrap();

    Json::<Sequence>(sequence.into())
}

async fn search_sequence(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>,
    pagination: Query<Pagination>
) -> impl IntoResponse {
    let client = get_meili_client();

    let sequence_index = client.index("sequences");

    let filter = format!(
        "langs IN [{}]",
        allowed_langs.join(", ")
    );

    let result = sequence_index
        .search()
        .with_query(&query)
        .with_filter(&filter)
        .with_offset(((pagination.page - 1) * pagination.size).try_into().unwrap())
        .with_limit(pagination.size.try_into().unwrap())
        .execute::<SequenceMeili>()
        .await
        .unwrap();

    let total = result.estimated_total_hits.unwrap();
    let sequence_ids: Vec<i32> = result.hits.iter().map(|a| a.result.id).collect();

    let mut sequences = db
        .sequence()
        .find_many(vec![
            sequence::id::in_vec(sequence_ids.clone())
        ])
        .exec()
        .await
        .unwrap();

    sequences.sort_by(|a, b| {
        let a_pos = sequence_ids.iter().position(|i| *i == a.id).unwrap();
        let b_pos: usize = sequence_ids.iter().position(|i| *i == b.id).unwrap();

        a_pos.cmp(&b_pos)
    });

    let page: Page<Sequence> = Page::new(
        sequences.iter().map(|item| item.clone().into()).collect(),
        total.try_into().unwrap(),
        &pagination
    );

    Json(page)
}

async fn get_sequence(
    db: Database,
    Path(sequence_id): Path<i32>
) -> impl IntoResponse {
    let sequence = db
        .sequence()
        .find_unique(
            sequence::id::equals(sequence_id)
        )
        .exec()
        .await
        .unwrap();

    match sequence {
        Some(sequence) => Json::<Sequence>(sequence.into()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_sequence_available_types(
    db: Database,
    Path(sequence_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>
) -> impl IntoResponse {
    let books = db
        .book()
        .find_many(vec![
            book::book_sequences::some(vec![
                book_sequence::sequence_id::equals(sequence_id)
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

async fn get_sequence_books(
    db: Database,
    Path(sequence_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<AllowedLangs>,
    pagination: Query<Pagination>
) -> impl IntoResponse {
    let sequence = db
        .sequence()
        .find_unique(
            sequence::id::equals(sequence_id)
        )
        .exec()
        .await
        .unwrap();

    let sequence = match sequence {
        Some(v) => v,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let books_count = db
        .book()
        .count(vec![
            book::book_sequences::some(vec![
                book_sequence::sequence_id::equals(sequence_id)
            ]),
            book::lang::in_vec(allowed_langs.clone())
        ])
        .exec()
        .await
        .unwrap();

    let books = db
        .book()
        .find_many(vec![
            book::book_sequences::some(vec![
                book_sequence::sequence_id::equals(sequence_id)
            ]),
            book::lang::in_vec(allowed_langs.clone())
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
            book::translations::fetch(vec![])
                .with(
                    translator::author::fetch()
                        .with(
                            author::author_annotation::fetch()
                        )
                )
        )
        .order_by(book::id::order(prisma_client_rust::Direction::Asc))
        .skip((pagination.page - 1) * pagination.size)
        .take(pagination.size)
        .exec()
        .await
        .unwrap();

    let page: PageWithParent<SequenceBook, Sequence> = PageWithParent::new(
        sequence.into(),
        books.iter().map(|item| item.clone().into()).collect(),
        books_count,
        &pagination
    );

    Json(page).into_response()
}


pub async fn get_sequences_router() -> Router {
    Router::new()
        .route("/random", get(get_random_sequence))
        .route("/search/:query", get(search_sequence))
        .route("/:sequence_id", get(get_sequence))
        .route("/:sequence_id/available_types", get(get_sequence_available_types))
        .route("/:sequence_id/books", get(get_sequence_books))
}
