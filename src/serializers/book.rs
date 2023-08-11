use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Serialize, Deserialize};

use crate::prisma::book::{self};

use super::{source::Source, utils::{get_available_types, get_translators, get_sequences, get_authors, get_genres}, author::Author, sequence::Sequence, genre::Genre};


fn default_langs() -> Vec<String> {
    vec![
        "ru".to_string(),
        "be".to_string(),
        "uk".to_string()
    ]
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

impl BookFilter {
    pub fn get_filter_vec(self) -> Vec<book::WhereParam> {
        let mut result = vec![];

        result.push(
            book::lang::in_vec(self.allowed_langs)
        );

        match self.is_deleted {
            Some(v) => {
                result.push(
                    book::is_deleted::equals(v)
                );
            },
            None => {
                result.push(
                    book::is_deleted::equals(false)
                );
            },
        };

        if let Some(uploaded_gte) = self.uploaded_gte {
            result.push(
                book::uploaded::gte(NaiveDateTime::new(uploaded_gte, NaiveTime::default()).and_utc().into())
            );
        };

        if let Some(uploaded_lte) = self.uploaded_lte {
            result.push(
                book::uploaded::lte(NaiveDateTime::new(uploaded_lte, NaiveTime::default()).and_utc().into())
            );
        };

        if let Some(id_gte) = self.id_gte {
            result.push(
                book::id::gte(id_gte)
            );
        };

        if let Some(id_lte) = self.id_lte {
            result.push(
                book::id::lte(id_lte)
            );
        };

        result
    }
}

#[derive(Serialize)]
pub struct RemoteBook {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub file_type: String,
    pub available_types: Vec<String>,
    pub uploaded: String,
    pub authors: Vec<Author>,
    pub translators: Vec<Author>,
    pub sequences: Vec<Sequence>,
    pub annotation_exists: bool,
    pub source: Source,
    pub remote_id: i32,
}

impl From<book::Data> for RemoteBook {
    fn from(value: book::Data) -> Self {
        let book::Data {
            id,
            title,
            lang,
            file_type,
            uploaded,
            book_authors,
            translations,
            book_sequences,
            book_annotation,
            source,
            remote_id,
            ..
        } = value;

        Self {
            id,
            title,
            lang,
            file_type: file_type.clone(),
            available_types: get_available_types(file_type, source.clone().unwrap().name),
            uploaded: uploaded.format("%Y-%m-%d").to_string(),
            authors: get_authors(book_authors),
            translators: get_translators(translations),
            sequences: get_sequences(book_sequences),
            annotation_exists: book_annotation.unwrap().is_some(),
            source: source.unwrap().as_ref().clone().into(),
            remote_id
        }
    }
}

#[derive(Serialize)]
pub struct BaseBook {
    pub id: i32,
    pub available_types: Vec<String>,
}

impl From<book::Data> for BaseBook {
    fn from(value: book::Data) -> Self {
        let book::Data {
            id,
            file_type,
            source,
            ..
        } = value;

        Self {
            id,
            available_types: get_available_types(file_type, source.clone().unwrap().name),
        }
    }
}

#[derive(Serialize)]
pub struct DetailBook {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub file_type: String,
    pub available_types: Vec<String>,
    pub uploaded: String,
    pub authors: Vec<Author>,
    pub translators: Vec<Author>,
    pub sequences: Vec<Sequence>,
    pub annotation_exists: bool,
    pub source: Source,
    pub remote_id: i32,
    pub genres: Vec<Genre>,
    pub is_deleted: bool,
    pub pages: Option<i32>
}

impl From<book::Data> for DetailBook {
    fn from(value: book::Data) -> Self {
        let book::Data {
            id,
            title,
            lang,
            file_type,
            uploaded,
            book_authors,
            translations,
            book_sequences,
            book_annotation,
            source,
            remote_id,
            book_genres,
            is_deleted,
            pages,
            ..
        } = value;

        Self {
            id,
            title,
            lang,
            file_type: file_type.clone(),
            available_types: get_available_types(file_type, source.clone().unwrap().name),
            uploaded: uploaded.format("%Y-%m-%d").to_string(),
            authors: get_authors(book_authors),
            translators: get_translators(translations),
            sequences: get_sequences(book_sequences),
            annotation_exists: book_annotation.unwrap().is_some(),
            source: source.unwrap().as_ref().clone().into(),
            remote_id,
            genres: get_genres(book_genres),
            is_deleted,
            pages,
        }
    }
}

#[derive(Deserialize)]
pub struct RandomBookFilter {
    pub allowed_langs: Vec<String>,
    pub genre: Option<i32>
}

#[derive(Serialize)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub file_type: String,
    pub available_types: Vec<String>,
    pub uploaded: String,
    pub authors: Vec<Author>,
    pub translators: Vec<Author>,
    pub sequences: Vec<Sequence>,
    pub annotation_exists: bool,
}

impl From<book::Data> for Book {
    fn from(value: book::Data) -> Self {
        let book::Data {
            id,
            title,
            lang,
            file_type,
            uploaded,
            book_authors,
            translations,
            book_sequences,
            book_annotation,
            source,
            ..
        } = value;

        Self {
            id,
            title,
            lang,
            file_type: file_type.clone(),
            available_types: get_available_types(file_type, source.clone().unwrap().name),
            uploaded: uploaded.format("%Y-%m-%d").to_string(),
            authors: get_authors(book_authors),
            translators: get_translators(translations),
            sequences: get_sequences(book_sequences),
            annotation_exists: book_annotation.unwrap().is_some(),
        }
    }
}
