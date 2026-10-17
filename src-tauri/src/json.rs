use serde::de::DeserializeOwned;

/// Parse JSON text leniently (json5: comments and trailing commas are allowed in hand-edited files)
pub fn parse<T: DeserializeOwned>(json_str: &str) -> Result<T, crate::error::AppError> {
    json5::from_str(json_str)
        .map_err(|e| crate::error::AppError::Validation(format!("Failed to parse JSON: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Deserialize)]
    struct Item {
        name: String,
        value: u32,
    }

    #[test]
    fn parses_valid_json() {
        let item: Item = parse(r#"{"name": "hello", "value": 42}"#).unwrap();
        assert_eq!(
            item,
            Item {
                name: "hello".to_string(),
                value: 42
            }
        );
    }

    #[test]
    fn accepts_comments_and_trailing_commas() {
        let item: Item = parse(
            "{ // note
 name: 'a', value: 1, }",
        )
        .unwrap();
        assert_eq!(item.value, 1);
    }

    #[test]
    fn reports_syntax_errors_as_validation_errors() {
        let msg = parse::<Item>(r#"{"name": "hello", "value":}"#)
            .unwrap_err()
            .to_string();
        assert!(msg.contains("Validation error"), "got: {}", msg);
    }

    #[test]
    fn rejects_missing_fields_and_wrong_types() {
        assert!(parse::<Item>(r#"{"name": "hello"}"#).is_err());
        assert!(parse::<Item>(r#"{"name": "hello", "value": "x"}"#).is_err());
        assert!(parse::<Item>("").is_err());
    }
}
