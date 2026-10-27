/// Trims and truncates a user-supplied search query to at most 256
/// *characters* (never slices bytes, so multi-byte UTF-8 is safe).
pub fn normalize_query(query: &str) -> String {
    query.trim().chars().take(256).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_long_cyrillic_query_without_panicking() {
        let input = format!("a{}", "я".repeat(200));
        let out = normalize_query(&input);
        assert_eq!(out.chars().count(), 201);
        assert_eq!(out, input);

        let input = format!("a{}", "я".repeat(400));
        let out = normalize_query(&input);
        assert_eq!(out.chars().count(), 256);
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(normalize_query("  abc \n"), "abc");
        assert_eq!(normalize_query("   "), "");
    }
}
