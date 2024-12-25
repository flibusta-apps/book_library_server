use serde::Serialize;

#[derive(sqlx::FromRow, sqlx::Type, Serialize)]
pub struct Source {
    pub id: i32,
    pub name: String,
}
