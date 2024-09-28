use std::collections::HashSet;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::{
    meilisearch::{get_meili_client, SequenceMeili},
    prisma::{author, book, book_author, book_sequence, sequence, translator},
    serializers::{
        allowed_langs::AllowedLangs,
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
) -> impl IntoResponse {
    let sequence_id = {
        let client = get_meili_client();

        let authors_index = client.index("sequences");

        let filter = format!("langs IN [{}]", allowed_langs.join(", "));

        get_random_item::<SequenceMeili>(authors_index, filter).await
    };

    let sequence = db
        .sequence()
        .find_unique(sequence::id::equals(sequence_id))
        .exec()
        .await
        .unwrap()
        .unwrap();

    Json::<Sequence>(sequence.into())
}

async fn search_sequence(
    db: Database,
    Path(query): Path<String>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let client = get_meili_client();

    let sequence_index = client.index("sequences");

    let filter = format!("langs IN [{}]", allowed_langs.join(", "));

    let result = sequence_index
        .search()
        .with_query(&query)
        .with_filter(&filter)
        .with_offset(
            ((pagination.page - 1) * pagination.size)
                .try_into()
                .unwrap(),
        )
        .with_limit(pagination.size.try_into().unwrap())
        .execute::<SequenceMeili>()
        .await
        .unwrap();

    let total = result.estimated_total_hits.unwrap();
    let sequence_ids: Vec<i32> = result.hits.iter().map(|a| a.result.id).collect();

    let mut sequences = db
        .sequence()
        .find_many(vec![sequence::id::in_vec(sequence_ids.clone())])
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
        &pagination,
    );

    Json(page)
}

async fn get_sequence(db: Database, Path(sequence_id): Path<i32>) -> impl IntoResponse {
    let sequence = db
        .sequence()
        .find_unique(sequence::id::equals(sequence_id))
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
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
) -> impl IntoResponse {
    let books = db
        .book()
        .find_many(vec![
            book::is_deleted::equals(false),
            book::book_sequences::some(vec![book_sequence::sequence_id::equals(sequence_id)]),
            book::lang::in_vec(allowed_langs),
        ])
        .exec()
        .await
        .unwrap();

    let mut file_types: HashSet<String> = HashSet::new();

    for book in books {
        file_types.insert(book.file_type.clone());
    }

    if file_types.contains("fb2") {
        file_types.insert("epub".to_string());
        file_types.insert("mobi".to_string());
        file_types.insert("fb2zip".to_string());
    }

    Json::<Vec<String>>(file_types.into_iter().collect())
}

async fn get_sequence_books(
    db: Database,
    Path(sequence_id): Path<i32>,
    axum_extra::extract::Query(AllowedLangs { allowed_langs }): axum_extra::extract::Query<
        AllowedLangs,
    >,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let sequence = db
        .sequence()
        .find_unique(sequence::id::equals(sequence_id))
        .exec()
        .await
        .unwrap();

    let sequence = match sequence {
        Some(v) => v,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let books_filter = vec![
        book_sequence::book::is(vec![
            book::is_deleted::equals(false),
            book::lang::in_vec(allowed_langs.clone()),
        ]),
        book_sequence::sequence_id::equals(sequence.id),
    ];

    let books_count = db
        .book_sequence()
        .count(books_filter.clone())
        .exec()
        .await
        .unwrap();

    let book_sequences = db
        .book_sequence()
        .find_many(vec![
            book_sequence::book::is(vec![
                book::is_deleted::equals(false),
                book::lang::in_vec(allowed_langs.clone()),
            ]),
            book_sequence::sequence_id::equals(sequence.id),
        ])
        .order_by(book_sequence::position::order(
            prisma_client_rust::Direction::Asc,
        ))
        .skip((pagination.page - 1) * pagination.size)
        .take(pagination.size)
        .exec()
        .await
        .unwrap();

    let book_ids: Vec<i32> = book_sequences.iter().map(|a| a.book_id).collect();

    let books = db
        .book()
        .find_many(vec![book::id::in_vec(book_ids)])
        .with(book::source::fetch())
        .with(book::book_annotation::fetch())
        .with(
            book::book_authors::fetch(vec![])
                .with(book_author::author::fetch().with(author::author_annotation::fetch())),
        )
        .with(
            book::translations::fetch(vec![])
                .with(translator::author::fetch().with(author::author_annotation::fetch())),
        )
        .with(book::book_sequences::fetch(vec![
            book_sequence::sequence_id::equals(sequence.id),
        ]))
        .order_by(book::id::order(prisma_client_rust::Direction::Asc))
        .exec()
        .await
        .unwrap();

    let mut sequence_books = books
        .iter()
        .map(|item| item.clone().into())
        .collect::<Vec<SequenceBook>>();

    sequence_books.sort_by(|a, b| a.position.cmp(&b.position));

    let page: PageWithParent<SequenceBook, Sequence> =
        PageWithParent::new(sequence.into(), sequence_books, books_count, &pagination);

    Json(page).into_response()
}

pub async fn get_sequences_router() -> Router {
    Router::new()
        .route("/random", get(get_random_sequence))
        .route("/search/:query", get(search_sequence))
        .route("/:sequence_id", get(get_sequence))
        .route(
            "/:sequence_id/available_types",
            get(get_sequence_available_types),
        )
        .route("/:sequence_id/books", get(get_sequence_books))
}
