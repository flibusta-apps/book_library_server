use serde::{Deserialize, Serialize};

use crate::prisma::genre;

use super::source::Source;

#[derive(Serialize)]
pub struct Genre {
    pub id: i32,
    pub source: Source,
    pub remote_id: i32,
    pub code: String,
    pub description: String,
    pub meta: String,
}

impl From<genre::Data> for Genre {
    fn from(val: genre::Data) -> Self {
        let genre::Data {
            id,
            remote_id,
            code,
            description,
            meta,
            source,
            ..
        } = val;

        Genre {
            id,
            remote_id,
            code,
            description,
            meta,
            source: source.unwrap().as_ref().clone().into(),
        }
    }
}

#[derive(Deserialize)]
pub struct GenreFilter {
    pub meta: Option<String>,
}
