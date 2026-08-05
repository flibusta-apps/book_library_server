use meilisearch_sdk::client::Client;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::config::CONFIG;

// Shared Meilisearch client, built once at first use and reused across all
// requests so the underlying reqwest client (and its TCP/TLS connection pool)
// isn't recreated on every handler call.
//
// LIMITATION: meilisearch-sdk 0.29.1's `Client::new`/`ReqwestClient::new` do not
// expose a way to plug in a custom `reqwest::Client` (the `ReqwestClient` field
// wrapping it is crate-private and there's no builder for HTTP timeouts), so we
// cannot configure a request timeout on this client here. If a hung Meilisearch
// instance needs to be bounded, wrap call sites with `tokio::time::timeout(...)`
// or upgrade the SDK once it supports a custom reqwest client/timeout.
pub static MEILI_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::new(&CONFIG.meili_host, Some(CONFIG.meili_master_key.clone()))
        .expect("Failed to build Meilisearch client")
});

pub trait GetId {
    fn get_id(&self) -> i32;
}

#[derive(Deserialize)]
pub struct AuthorMeili {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: String,
    pub author_langs: Vec<String>,
    pub translator_langs: Vec<String>,
    pub books_count: i32,
}

impl GetId for AuthorMeili {
    fn get_id(&self) -> i32 {
        self.id
    }
}

#[derive(Deserialize)]
pub struct BookMeili {
    pub id: i32,
    pub title: String,
    pub lang: String,
    pub genres: Vec<i32>,
}

impl GetId for BookMeili {
    fn get_id(&self) -> i32 {
        self.id
    }
}

#[derive(Deserialize)]
pub struct GenreMeili {
    pub id: i32,
    pub description: String,
    pub meta: String,
    pub langs: Vec<String>,
    pub books_count: i32,
}

impl GetId for GenreMeili {
    fn get_id(&self) -> i32 {
        self.id
    }
}

#[derive(Deserialize)]
pub struct SequenceMeili {
    pub id: i32,
    pub name: String,
    pub langs: Vec<String>,
    pub books_count: i32,
}

impl GetId for SequenceMeili {
    fn get_id(&self) -> i32 {
        self.id
    }
}
