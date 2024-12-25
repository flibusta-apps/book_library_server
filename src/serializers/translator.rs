use chrono::NaiveDate;
use serde::Serialize;

use super::date::naive_date_serializer;

use super::{author::Author, sequence::Sequence};

#[derive(sqlx::FromRow, Serialize)]
pub struct TranslatorBook {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub file_type: String,
    pub year: i32,
    pub available_types: Vec<String>,
    #[serde(serialize_with = "naive_date_serializer::serialize")]
    pub uploaded: NaiveDate,
    pub authors: Vec<Author>,
    pub sequences: Vec<Sequence>,
    pub annotation_exists: bool,
}
