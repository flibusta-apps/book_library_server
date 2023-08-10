use serde::Serialize;

use crate::prisma::book;

use super::{author::Author, sequence::Sequence, utils::{get_available_types, get_authors, get_sequences}};

#[derive(Serialize)]
pub struct TranslatorBook {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub file_type: String,
    pub available_types: Vec<String>,
    pub uploaded: String,
    pub authors: Vec<Author>,
    pub sequences: Vec<Sequence>,
    pub annotation_exists: bool,
}

impl From<book::Data> for TranslatorBook {
    fn from(val: book::Data) -> Self {
        let book::Data {
            id,
            title,
            lang,
            file_type,
            uploaded,
            book_authors,
            book_sequences,
            book_annotation,
            source,
            ..
        } = val;

        TranslatorBook {
            id,
            title,
            lang,
            file_type: file_type.clone(),
            available_types: get_available_types(file_type.clone(), source.unwrap().name),
            uploaded: uploaded.format("%Y-%m-%d").to_string(),
            authors: get_authors(book_authors),
            sequences: get_sequences(book_sequences),
            annotation_exists: book_annotation.unwrap().is_some(),
        }
    }
}
