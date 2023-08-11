use crate::prisma::{translator, book_sequence, book_author, book_genre};

use super::{author::Author, sequence::Sequence, genre::Genre};

pub fn get_available_types(file_type: String, source_name: String) -> Vec<String> {
    if file_type == "fb2" && source_name == "flibusta" {
        vec![
            "fb2".to_string(),
            "fb2zip".to_string(),
            "epub".to_string(),
            "mobi".to_string(),
        ]
    } else {
        vec![file_type]
    }
}

pub fn get_authors(
    book_authors: Option<Vec<book_author::Data>>
) -> Vec<Author> {
    book_authors
        .unwrap()
        .iter()
        .map(|item| item.author.clone().unwrap().as_ref().clone().into())
        .collect()
}

pub fn get_translators(
    translations: Option<Vec<translator::Data>>
) -> Vec<Author> {
    translations
        .unwrap()
        .iter()
        .map(|item| item.author.clone().unwrap().as_ref().clone().into())
        .collect()
}

pub fn get_sequences(
    book_sequences: Option<Vec<book_sequence::Data>>
) -> Vec<Sequence> {
    book_sequences
        .unwrap()
        .iter()
        .map(|item| item.sequence.clone().unwrap().as_ref().clone().into())
        .collect()
}

pub fn get_genres(
    book_genres: Option<Vec<book_genre::Data>>
) -> Vec<Genre> {
    book_genres
        .unwrap()
        .iter()
        .map(|item| item.genre.clone().unwrap().as_ref().clone().into())
        .collect()
}
