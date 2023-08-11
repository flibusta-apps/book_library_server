use serde::Deserialize;

#[derive(Deserialize)]
pub struct AllowedLangs {
    pub allowed_langs: Vec<String>
}
