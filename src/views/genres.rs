use std::collections::HashSet;

use axum::{Router, routing::get, extract::Query, Json, response::IntoResponse};
use prisma_client_rust::Direction;

use crate::{serializers::{pagination::{Pagination, Page}, genre::{Genre, GenreFilter}}, prisma::genre};

use super::Database;


pub async fn get_genres(
    db: Database,
    pagination: Query<Pagination>,
    Query(GenreFilter { meta }): Query<GenreFilter>
) -> impl IntoResponse {
    let filter = {
        match meta {
            Some(meta) => vec![
                genre::meta::equals(meta)
            ],
            None => vec![],
        }
    };

    let genres_count = db
        .genre()
        .count(filter.clone())
        .exec()
        .await
        .unwrap();

    let genres = db
        .genre()
        .find_many(filter)
        .with(
            genre::source::fetch()
        )
        .order_by(genre::id::order(Direction::Asc))
        .skip((pagination.page - 1) * pagination.size)
        .take(pagination.size)
        .exec()
        .await
        .unwrap();

    let page: Page<Genre> = Page::new(
        genres.iter().map(|item| item.clone().into()).collect(),
        genres_count,
        &pagination
    );

    Json(page)
}


pub async fn get_genre_metas(
    db: Database
) -> impl IntoResponse {
    let genres = db
        .genre()
        .find_many(vec![])
        .order_by(genre::id::order(Direction::Asc))
        .exec()
        .await
        .unwrap();

    let mut metas: HashSet<String> = HashSet::new();

    for genre in genres {
        metas.insert(genre.meta.clone());
    }

    Json::<Vec<String>>(metas.into_iter().collect())
}


pub async fn get_genres_router() -> Router {
    Router::new()
        .route("/", get(get_genres))
        .route("/metas", get(get_genre_metas))
}