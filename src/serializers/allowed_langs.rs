use serde::Deserialize;

pub(crate) fn default_langs() -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn deserialize(value: serde_json::Value) -> Result<AllowedLangs, serde_json::Error> {
        serde_json::from_value(value)
    }

    #[test]
    fn empty_input_uses_defaults() {
        let allowed_langs = deserialize(serde_json::json!({})).expect("should deserialize");
        assert_eq!(allowed_langs.allowed_langs, default_langs());
    }

    #[test]
    fn valid_known_langs_pass_through() {
        let allowed_langs = deserialize(serde_json::json!({ "allowed_langs": ["ru", "en"] }))
            .expect("valid 2-letter codes should deserialize");
        assert_eq!(allowed_langs.allowed_langs, vec!["ru", "en"]);
    }

    #[test]
    fn valid_three_letter_lang_passes() {
        let allowed_langs = deserialize(serde_json::json!({ "allowed_langs": ["chu"] }))
            .expect("valid 3-letter codes should deserialize");
        assert_eq!(allowed_langs.allowed_langs, vec!["chu"]);
    }

    #[test]
    fn uppercase_lang_is_rejected() {
        let result = deserialize(serde_json::json!({ "allowed_langs": ["RU"] }));
        assert!(result.is_err());
    }

    #[test]
    fn too_short_lang_is_rejected() {
        let result = deserialize(serde_json::json!({ "allowed_langs": ["r"] }));
        assert!(result.is_err());
    }

    #[test]
    fn too_long_lang_is_rejected() {
        let result = deserialize(serde_json::json!({ "allowed_langs": ["russ"] }));
        assert!(result.is_err());
    }

    #[test]
    fn injection_attempt_via_filter_syntax_is_rejected() {
        // These values would otherwise be interpolated directly into a
        // Meilisearch filter expression like `lang IN [{}]`; the validator
        // must reject anything that isn't a plain lowercase ASCII code.
        let malicious_values = [
            "ru] OR 1=1 OR lang IN [ru",
            "ru, be",
            "ru' OR '1'='1",
            "",
            "ru123",
        ];

        for value in malicious_values {
            let result = deserialize(serde_json::json!({ "allowed_langs": [value] }));
            assert!(result.is_err(), "expected `{value}` to be rejected");
        }
    }
}
