use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct BookAnnotation {
    pub id: i32,
    pub title: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}
