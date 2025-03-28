use chrono::NaiveDate;
use serde::Serialize;

use super::author::Author;
use super::date::naive_date_serializer;

#[derive(sqlx::FromRow, sqlx::Type, Serialize)]
#[sqlx(type_name = "sequence_type")]
pub struct Sequence {
    pub id: i32,
    pub name: String,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct SequenceBook {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub file_type: String,
    pub year: i32,
    pub available_types: Vec<String>,
    #[serde(serialize_with = "naive_date_serializer::serialize")]
    pub uploaded: NaiveDate,
    pub authors: Vec<Author>,
    pub translators: Vec<Author>,
    pub annotation_exists: bool,
    pub position: i32,
}
