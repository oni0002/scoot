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
