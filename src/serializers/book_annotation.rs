use serde::Serialize;

use crate::prisma::book_annotation;

#[derive(Serialize)]
pub struct BookAnnotation {
    pub id: i32,
    pub title: String,
    pub text: String,
    pub file: Option<String>,
}

impl From<book_annotation::Data> for BookAnnotation {
    fn from(value: book_annotation::Data) -> Self {
        let book_annotation::Data {
            id,
            title,
            text,
            file,
            ..
        } = value;

        Self {
            id,
            title,
            text,
            file,
        }
    }
}
