use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

/// Parse JSON string with json5
fn parse<T: DeserializeOwned>(json_str: &str) -> Result<T, crate::error::AppError> {
    json5::from_str(json_str)
        .map_err(|e| crate::error::AppError::Validation(format!("Failed to parse JSON: {}", e)))
}

/// Validate data against its JSON Schema definition
pub fn validate<T: Serialize + JsonSchema>(data: &T) -> Result<(), crate::error::AppError> {
    let normalized = serde_json::to_value(data).map_err(|e| {
        crate::error::AppError::Validation(format!("Failed to serialize data: {}", e))
    })?;

    let schema_val = serde_json::to_value(schemars::schema_for!(T)).unwrap_or_default();
    let validator = jsonschema::validator_for(&schema_val).map_err(|e| {
        crate::error::AppError::Validation(format!("Failed to compile schema: {}", e))
    })?;

    let errors: Vec<String> = validator
        .iter_errors(&normalized)
        .map(|err| format!("Error at {}: {}", err.instance_path, err))
        .collect();

    if !errors.is_empty() {
        return Err(crate::error::AppError::Validation(format!(
            "Schema validation failed: {}",
            errors.join(", ")
        )));
    }

    Ok(())
}

/// Parse JSON string with json5 and validate with schema
pub fn parse_and_validate<T>(json_str: &str) -> Result<T, crate::error::AppError>
where
    T: DeserializeOwned + Serialize + JsonSchema,
{
    let data: T = parse(json_str)?;
    validate(&data)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
    struct Item {
        name: String,
        value: u32,
    }

    #[test]
    fn parse_and_validate_valid_json() {
        let json = r#"{"name": "hello", "value": 42}"#;
        let result: Result<Item, _> = parse_and_validate(json);
        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.name, "hello");
        assert_eq!(item.value, 42);
    }

    #[test]
    fn parse_and_validate_syntax_error() {
        let json = r#"{"name": "hello", "value":}"#;
        let result: Result<Item, _> = parse_and_validate(json);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Validation error"), "expected Validation error, got: {}", msg);
    }

    #[test]
    fn parse_and_validate_missing_required_field() {
        let json = r#"{"name": "hello"}"#;
        let result: Result<Item, _> = parse_and_validate(json);
        assert!(result.is_err());
    }

    #[test]
    fn parse_and_validate_wrong_type() {
        let json = r#"{"name": "hello", "value": "not_a_number"}"#;
        let result: Result<Item, _> = parse_and_validate(json);
        assert!(result.is_err());
    }

    #[test]
    fn validate_valid_data() {
        let item = Item { name: "test".to_string(), value: 0 };
        assert!(validate(&item).is_ok());
    }

    #[test]
    fn parse_and_validate_empty_string() {
        let result: Result<Item, _> = parse_and_validate("");
        assert!(result.is_err());
    }
}
