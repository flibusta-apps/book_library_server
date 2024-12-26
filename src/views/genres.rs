use std::collections::HashSet;

use axum::{extract::Query, response::IntoResponse, routing::get, Json, Router};

use crate::serializers::{
    genre::{Genre, GenreFilter},
    pagination::{Page, Pagination},
};

use crate::serializers::source::Source;

use super::Database;

pub async fn get_genres(
    db: Database,
    pagination: Query<Pagination>,
    Query(GenreFilter { meta }): Query<GenreFilter>,
) -> impl IntoResponse {
    let genres_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM genres
        WHERE (meta = $1 OR $1 IS NULL)
        "#,
        meta
    )
    .fetch_one(&db.0)
    .await
    .unwrap()
    .unwrap();

    let genres = sqlx::query_as!(
        Genre,
        r#"
        SELECT
            genres.id,
            genres.remote_id,
            genres.code,
            genres.description,
            genres.meta,
            (
                SELECT
                    ROW(
                        sources.id,
                        sources.name
                    )::source_type
                FROM sources
                WHERE sources.id = genres.source
            ) AS "source!: Source"
        FROM genres
        WHERE (meta = $1 OR $1 IS NULL)
        ORDER BY genres.id ASC
        LIMIT $2 OFFSET $3
        "#,
        meta,
        pagination.size,
        (pagination.page - 1) * pagination.size
    )
    .fetch_all(&db.0)
    .await
    .unwrap();

    let page: Page<Genre> = Page::new(genres, genres_count, &pagination);

    Json(page)
}

pub async fn get_genre_metas(db: Database) -> impl IntoResponse {
    let genres = sqlx::query_as!(
        Genre,
        r#"
        SELECT
            genres.id,
            genres.remote_id,
            genres.code,
            genres.description,
            genres.meta,
            (
                SELECT
                    ROW(
                        sources.id,
                        sources.name
                    )::source_type
                FROM sources
                WHERE sources.id = genres.source
            ) AS "source!: Source"
        FROM genres
        ORDER BY genres.id ASC
        "#
    )
    .fetch_all(&db.0)
    .await
    .unwrap();

    let mut metas: HashSet<String> = HashSet::new();

    for genre in genres {
        metas.insert(genre.meta.clone());
    }

    let mut metas: Vec<String> = metas.into_iter().collect();
    metas.sort();

    Json::<Vec<String>>(metas)
}

pub async fn get_genres_router() -> Router {
    Router::new()
        .route("/", get(get_genres))
        .route("/metas", get(get_genre_metas))
}
