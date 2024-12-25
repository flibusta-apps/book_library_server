use serde::{Deserialize, Serialize};

use super::source::Source;

#[derive(sqlx::FromRow, sqlx::Type, Serialize)]
pub struct Genre {
    pub id: i32,
    pub source: Source,
    pub remote_id: i32,
    pub code: String,
    pub description: String,
    pub meta: String,
}

#[derive(Deserialize)]
pub struct GenreFilter {
    pub meta: Option<String>,
}
