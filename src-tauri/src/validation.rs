use jsonschema::Validator;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

/// Compile the JSON Schema of `T`. Compiling is costly, so callers keep the result in a static
/// (`LazyLock<Validator>`) instead of compiling on every load.
pub fn compile<T: JsonSchema>() -> Validator {
    let schema = serde_json::to_value(schemars::schema_for!(T)).unwrap_or_default();
    jsonschema::validator_for(&schema).expect("a schema derived by schemars must compile")
}

/// Parse JSON string with json5
pub fn parse<T: DeserializeOwned>(json_str: &str) -> Result<T, crate::error::AppError> {
    json5::from_str(json_str)
        .map_err(|e| crate::error::AppError::Validation(format!("Failed to parse JSON: {}", e)))
}

/// Validate data against a compiled JSON Schema
pub fn validate<T: Serialize>(
    validator: &Validator,
    data: &T,
) -> Result<(), crate::error::AppError> {
    let normalized = serde_json::to_value(data).map_err(|e| {
        crate::error::AppError::Validation(format!("Failed to serialize data: {}", e))
    })?;

    let errors: Vec<String> = validator
        .iter_errors(&normalized)
        .map(|err| format!("Error at {}: {}", err.instance_path(), err))
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
pub fn parse_and_validate<T>(
    json_str: &str,
    validator: &Validator,
) -> Result<T, crate::error::AppError>
where
    T: DeserializeOwned + Serialize,
{
    let data: T = parse(json_str)?;
    validate(validator, &data)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::sync::LazyLock;

    static ITEM_SCHEMA: LazyLock<Validator> = LazyLock::new(compile::<Item>);

    fn parse_and_validate(json: &str) -> Result<Item, crate::error::AppError> {
        super::parse_and_validate(json, &ITEM_SCHEMA)
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
    struct Item {
        name: String,
        value: u32,
    }

    #[test]
    fn parse_and_validate_valid_json() {
        let json = r#"{"name": "hello", "value": 42}"#;
        let result = parse_and_validate(json);
        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.name, "hello");
        assert_eq!(item.value, 42);
    }

    #[test]
    fn parse_and_validate_syntax_error() {
        let json = r#"{"name": "hello", "value":}"#;
        let result = parse_and_validate(json);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Validation error"),
            "expected Validation error, got: {}",
            msg
        );
    }

    #[test]
    fn parse_and_validate_missing_required_field() {
        let json = r#"{"name": "hello"}"#;
        let result = parse_and_validate(json);
        assert!(result.is_err());
    }

    #[test]
    fn parse_and_validate_wrong_type() {
        let json = r#"{"name": "hello", "value": "not_a_number"}"#;
        let result = parse_and_validate(json);
        assert!(result.is_err());
    }

    #[test]
    fn validate_valid_data() {
        let item = Item {
            name: "test".to_string(),
            value: 0,
        };
        assert!(validate(&ITEM_SCHEMA, &item).is_ok());
    }

    #[test]
    fn parse_and_validate_empty_string() {
        let result = parse_and_validate("");
        assert!(result.is_err());
    }
}
