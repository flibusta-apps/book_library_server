pub fn get_available_types(file_type: String, source_name: String) -> Vec<String> {
    if file_type == "fb2" && source_name == "flibusta" {
        vec![
            "fb2".to_string(),
            "fb2zip".to_string(),
            "epub".to_string(),
            "mobi".to_string(),
        ]
    } else {
        vec![file_type]
    }
}
