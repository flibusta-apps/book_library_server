use serde::Serialize;

use crate::prisma::{author, book};

use super::{
    sequence::Sequence,
    utils::{get_available_types, get_sequences, get_translators},
};

#[derive(Serialize)]
pub struct Author {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: String,
    pub annotation_exists: bool,
}

impl From<author::Data> for Author {
    fn from(val: author::Data) -> Self {
        let author::Data {
            id,
            first_name,
            last_name,
            middle_name,
            author_annotation,
            ..
        } = val;

        Author {
            id,
            first_name,
            last_name,
            middle_name: middle_name.unwrap_or("".to_string()),
            annotation_exists: author_annotation.unwrap().is_some(),
        }
    }
}

#[derive(Serialize)]
pub struct AuthorBook {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub file_type: String,
    pub year: i32,
    pub available_types: Vec<String>,
    pub uploaded: String,
    pub translators: Vec<Author>,
    pub sequences: Vec<Sequence>,
    pub annotation_exists: bool,
}

impl From<book::Data> for AuthorBook {
    fn from(val: book::Data) -> Self {
        let book::Data {
            id,
            title,
            lang,
            file_type,
            year,
            uploaded,
            translations,
            book_sequences,
            book_annotation,
            source,
            ..
        } = val;

        AuthorBook {
            id,
            title,
            lang,
            file_type: file_type.clone(),
            year,
            available_types: get_available_types(file_type, source.unwrap().name),
            uploaded: uploaded.format("%Y-%m-%d").to_string(),
            translators: get_translators(translations),
            sequences: get_sequences(book_sequences),
            annotation_exists: book_annotation.unwrap().is_some(),
        }
    }
}
