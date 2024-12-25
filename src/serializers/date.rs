use chrono::NaiveDate;
use serde::Serializer;

const FORMAT: &str = "%Y-%m-%d";

pub mod naive_date_serializer {
    use super::*;

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted_date = date.format(FORMAT).to_string();
        serializer.serialize_str(&formatted_date)
    }
}
