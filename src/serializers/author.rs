use chrono::NaiveDate;
use serde::Serialize;

use super::date::naive_date_serializer;
use super::sequence::Sequence;

#[derive(sqlx::FromRow, sqlx::Type, Serialize)]
pub struct Author {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: String,
    pub annotation_exists: bool,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct AuthorBook {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub file_type: String,
    pub year: i32,
    pub available_types: Vec<String>,
    #[serde(serialize_with = "naive_date_serializer::serialize")]
    pub uploaded: NaiveDate,
    pub translators: sqlx::types::Json<Vec<Author>>,
    pub sequences: sqlx::types::Json<Vec<Sequence>>,
    pub annotation_exists: bool,
}
