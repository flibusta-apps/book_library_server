use serde::Serialize;

use crate::prisma::author_annotation;

#[derive(Serialize)]
pub struct AuthorAnnotation {
    pub id: i32,
    pub title: String,
    pub text: String,
    pub file: Option<String>,
}

impl From<author_annotation::Data> for AuthorAnnotation {
    fn from(val: author_annotation::Data) -> Self {
        let author_annotation::Data {
            id,
            title,
            text,
            file,
            ..
        } = val;

        AuthorAnnotation {
            id,
            title,
            text,
            file,
        }
    }
}
