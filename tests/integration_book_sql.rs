//! DB regression tests for runtime-decoded book queries used by search and
//! random. `books.year` is SMALLINT while the serializers use `i32`; runtime
//! `query_as` must cast it (a missing cast caused HTTP 500 on every search).

use book_library_server::views::book_sql::{random_book_candidate, search_books_by_ids};
use sqlx::PgPool;

async fn seed(pool: &PgPool) -> (i32, i32, i32) {
    sqlx::query("INSERT INTO sources (id, name) VALUES (1, 'test-source')")
        .execute(pool)
        .await
        .unwrap();
    let mut ids = Vec::new();
    for (remote_id, lang, deleted) in [(1, "ru", false), (2, "ru", true), (3, "en", false)] {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO books (source, remote_id, title, lang, file_type, uploaded, is_deleted, year)
             VALUES (1, $1, 'Book', $2, 'fb2', CURRENT_DATE, $3, 2020) RETURNING id",
        )
        .bind(remote_id)
        .bind(lang)
        .bind(deleted)
        .fetch_one(pool)
        .await
        .unwrap();
        ids.push(id);
    }
    (ids[0], ids[1], ids[2])
}

#[sqlx::test(migrations = "./migrations")]
async fn search_hydration_decodes_and_filters(pool: PgPool) {
    let (ok, deleted, english) = seed(&pool).await;
    let langs = vec!["ru".to_string()];
    let books = search_books_by_ids(&pool, &[ok, deleted, english], &langs)
        .await
        .expect("search hydration must decode SMALLINT year");
    assert_eq!(books.iter().map(|b| b.id).collect::<Vec<_>>(), vec![ok]);
    assert_eq!(books[0].year, 2020);
}

#[sqlx::test(migrations = "./migrations")]
async fn random_candidate_decodes_and_filters(pool: PgPool) {
    let (ok, deleted, english) = seed(&pool).await;
    let langs = vec!["ru".to_string()];
    let book = random_book_candidate(&pool, ok, &langs)
        .await
        .expect("random candidate must decode")
        .expect("visible book");
    assert_eq!(book.year, 2020);
    assert!(random_book_candidate(&pool, deleted, &langs)
        .await
        .unwrap()
        .is_none());
    assert!(random_book_candidate(&pool, english, &langs)
        .await
        .unwrap()
        .is_none());
}
