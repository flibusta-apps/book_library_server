use serde::Deserialize;

fn default_langs() -> Vec<String> {
    vec!["ru".to_string(), "be".to_string(), "uk".to_string()]
}

/// Deserializes a list of language codes, validating that each entry
/// matches the ISO 639 language code shape: 2-3 lowercase ASCII letters.
///
/// This guards against Meilisearch filter injection, since these values
/// are later interpolated into filter expressions like
/// `format!("lang IN [{}]", allowed_langs.join(", "))`.
pub fn deserialize_lang_codes<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let langs: Vec<String> = Vec::deserialize(deserializer)?;

    for lang in &langs {
        let is_valid = matches!(lang.len(), 2 | 3) && lang.chars().all(|c| c.is_ascii_lowercase());

        if !is_valid {
            return Err(serde::de::Error::custom(format!(
                "invalid language code: {lang}"
            )));
        }
    }

    Ok(langs)
}

#[derive(Deserialize)]
pub struct AllowedLangs {
    #[serde(default = "default_langs", deserialize_with = "deserialize_lang_codes")]
    pub allowed_langs: Vec<String>,
}
