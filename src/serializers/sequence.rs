use serde::Serialize;

use crate::prisma::{book, sequence};

use super::{
    author::Author,
    utils::{get_authors, get_available_types, get_translators},
};

#[derive(Serialize)]
pub struct Sequence {
    pub id: i32,
    pub name: String,
}

impl From<sequence::Data> for Sequence {
    fn from(val: sequence::Data) -> Self {
        let sequence::Data { id, name, .. } = val;

        Sequence { id, name }
    }
}

#[derive(Serialize)]
pub struct SequenceBook {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub file_type: String,
    pub year: i32,
    pub available_types: Vec<String>,
    pub uploaded: String,
    pub authors: Vec<Author>,
    pub translators: Vec<Author>,
    pub annotation_exists: bool,
    pub position: i32,
}

impl From<book::Data> for SequenceBook {
    fn from(value: book::Data) -> Self {
        let book::Data {
            id,
            title,
            lang,
            file_type,
            year,
            uploaded,
            book_authors,
            translations,
            book_annotation,
            source,
            book_sequences,
            ..
        } = value;

        Self {
            id,
            title,
            lang,
            file_type: file_type.clone(),
            year,
            available_types: get_available_types(file_type, source.clone().unwrap().name),
            uploaded: uploaded.format("%Y-%m-%d").to_string(),
            authors: get_authors(book_authors),
            translators: get_translators(translations),
            annotation_exists: book_annotation.unwrap().is_some(),
            position: book_sequences.unwrap().first().unwrap().position,
        }
    }
}
