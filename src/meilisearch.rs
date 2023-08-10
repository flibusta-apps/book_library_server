use meilisearch_sdk::Client;
use serde::Deserialize;

use crate::config::CONFIG;


pub fn get_meili_client() -> Client {
    Client::new(
        &CONFIG.meili_host,
        Some(CONFIG.meili_master_key.clone())
    )
}

#[derive(Deserialize)]
pub struct AuthorMeili {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: String,
    pub author_langs: Vec<String>,
    pub translator_langs: Vec<String>,
    pub books_count: i32
}

#[derive(Deserialize)]
pub struct BookMeili {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub genres: Vec<i32>
}

#[derive(Deserialize)]
pub struct GenreMeili {
    pub id: i32,
    pub description: String,
    pub meta: String,
    pub langs: Vec<String>,
    pub books_count: i32
}

#[derive(Deserialize)]
pub struct SequenceMeili {
    pub id: i32,
    pub name: String,
    pub langs: Vec<String>,
    pub books_count: i32
}
