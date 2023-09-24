use serde::Deserialize;

fn default_langs() -> Vec<String> {
    vec!["ru".to_string(), "be".to_string(), "uk".to_string()]
}

#[derive(Deserialize)]
pub struct AllowedLangs {
    #[serde(default = "default_langs")]
    pub allowed_langs: Vec<String>,
}
