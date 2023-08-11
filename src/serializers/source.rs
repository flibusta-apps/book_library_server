use serde::Serialize;

use crate::prisma::source;

#[derive(Serialize)]
pub struct Source {
    pub id: i32,
    pub name: String
}

impl From<source::Data> for Source 
{
    fn from(val: source::Data) -> Self {
        let source::Data {
            id,
            name,
            ..
        } = val;

        Source {
            id,
            name
        }
    }
}