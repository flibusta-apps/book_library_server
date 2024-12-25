use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::date::naive_date_serializer;

use super::{author::Author, genre::Genre, sequence::Sequence, source::Source};

fn default_langs() -> Vec<String> {
    vec!["ru".to_string(), "be".to_string(), "uk".to_string()]
}

#[derive(Deserialize)]
pub struct BookFilter {
    #[serde(default = "default_langs")]
    pub allowed_langs: Vec<String>,
    pub is_deleted: Option<bool>,
    pub uploaded_gte: Option<NaiveDate>,
    pub uploaded_lte: Option<NaiveDate>,
    pub id_gte: Option<i32>,
    pub id_lte: Option<i32>,
}

#[derive(Serialize)]
pub struct RemoteBook {
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
    pub sequences: Vec<Sequence>,
    pub annotation_exists: bool,
    pub source: Source,
    pub remote_id: i32,
}

#[derive(Serialize)]
pub struct BaseBook {
    pub id: i32,
    pub available_types: Vec<String>,
}

#[derive(Serialize)]
pub struct DetailBook {
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
    pub sequences: Vec<Sequence>,
    pub annotation_exists: bool,
    pub source: Source,
    pub remote_id: i32,
    pub genres: Vec<Genre>,
    pub is_deleted: bool,
    pub pages: Option<i32>,
}

#[derive(Deserialize)]
pub struct RandomBookFilter {
    pub allowed_langs: Vec<String>,
    pub genre: Option<i32>,
}

#[derive(Serialize)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub file_type: String,
    pub year: i32,
    pub available_types: Vec<String>,
    #[serde(serialize_with = "naive_date_serializer::serialize")]
    pub uploaded: NaiveDate,
    pub authors: sqlx::types::Json<Vec<Author>>,
    pub translators: sqlx::types::Json<Vec<Author>>,
    pub sequences: sqlx::types::Json<Vec<Sequence>>,
    pub annotation_exists: bool,
}
