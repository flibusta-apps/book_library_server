use serde::Serialize;

#[derive(sqlx::FromRow, sqlx::Type, Serialize)]
#[sqlx(type_name = "source_type")]
pub struct Source {
    pub id: i32,
    pub name: String,
}
