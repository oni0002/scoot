use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

/// Parse JSON string with json5
fn parse<T: DeserializeOwned>(json_str: &str) -> Result<T, crate::error::AppError> {
    json5::from_str(json_str)
        .map_err(|e| crate::error::AppError::Validation(format!("Failed to parse JSON: {}", e)))
}

/// Validate data against its JSON Schema definition
pub fn validate<T: Serialize + JsonSchema>(data: &T) -> Result<(), crate::error::AppError> {
    // Convert to JSON Value for validation
    let normalized = serde_json::to_value(data).map_err(|e| {
        crate::error::AppError::Validation(format!("Failed to serialize data: {}", e))
    })?;

    // Generate schema from type T and validate
    let schema_val = serde_json::to_value(schemars::schema_for!(T)).unwrap_or_default();
    // Compile schema
    let compiled = jsonschema::JSONSchema::compile(&schema_val).map_err(|e| {
        crate::error::AppError::Validation(format!("Failed to compile schema: {}", e))
    })?;

    // Validate data against schema
    if let Err(errors) = compiled.validate(&normalized) {
        let msgs: Vec<String> = errors
            .map(|err| format!("Error at {}: {}", err.instance_path, err))
            .collect();
        return Err(crate::error::AppError::Validation(format!(
            "Schema validation failed: {}",
            msgs.join(", ")
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
