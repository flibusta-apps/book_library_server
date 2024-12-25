use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct AuthorAnnotation {
    pub id: i32,
    pub title: String,
    pub text: String,
    pub file: Option<String>,
}
