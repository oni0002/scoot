use super::command::{Command, Commands};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

pub const DEFAULT_THEME: &str = "dark";
pub const DEFAULT_SHORTCUT: &str = "Alt+Space";

/// Config struct
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[schemars(range(min = 1, max = 100))]
    // TODO: Remove `alias = "max_results"` in v1.0.0 (Legacy config support)
    #[serde(alias = "max_results")]
    pub max_results: usize,
    #[schemars(range(min = 0.0, max = 1.0))]
    // TODO: Remove `alias = "fuzzy_threshold"` in v1.0.0 (Legacy config support)
    #[serde(alias = "fuzzy_threshold")]
    pub fuzzy_threshold: f64,
    pub bookmarks: BookmarkConfig,
    pub applications: ApplicationConfig,
    #[schemars(regex(
        pattern = r"^(light|dark|cupcake|bumblebee|emerald|corporate|synthwave|retro|cyberpunk|valentine|halloween|garden|forest|aqua|lofi|pastel|fantasy|wireframe|black|luxury|dracula|cmyk|autumn|business|acid|lemonade|night|coffee|winter|dim|nord|sunset)$"
    ))]
    pub theme: String,
    #[schemars(regex(
        pattern = r"^((Cmd|Command|Ctrl|Control|Alt|Shift|Super|Option|cmd|command|ctrl|control|alt|shift|super|option)\+)+([A-Z0-9a-z]|Space|Enter|Tab|F[1-9]|F1[0-2]|space|enter|tab|f[1-9]|f1[0-2])$"
    ))]
    pub hotkey: String,
}

/// Bookmark config struct
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkConfig {
    pub enabled: bool,
    pub browser: String,
    pub prompt: Option<String>,
    #[schemars(range(min = 1))]
    // TODO: Remove `alias = "refresh_interval_minutes"` in v1.0.0 (Legacy config support)
    #[serde(alias = "refresh_interval_minutes")]
    pub refresh_interval_minutes: u64,
}

/// Application config struct
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationConfig {
    pub enabled: bool,
    pub directories: Vec<String>,
    pub extensions: Vec<String>,
}

/// ApplicationConfig default values
impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            directories: vec![
                r"%APPDATA%\Microsoft\Windows\Start Menu\Programs".to_string(),
                r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs".to_string(),
            ],
            extensions: vec!["lnk".to_string()],
        }
    }
}

/// BookmarkConfig default values
impl Default for BookmarkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            browser: "brave".to_string(),
            prompt: None,
            refresh_interval_minutes: 30,
        }
    }
}

/// Config default values
impl Default for Config {
    fn default() -> Self {
        Self {
            max_results: 10,
            fuzzy_threshold: 0.5,
            bookmarks: BookmarkConfig::default(),
            applications: ApplicationConfig::default(),
            theme: DEFAULT_THEME.to_string(),
            hotkey: DEFAULT_SHORTCUT.to_string(),
        }
    }
}

/// Config schema validation
impl Config {
    /// Generate Config JSON schema
    pub fn generate_schema() -> serde_json::Value {
        let schema = schema_for!(Config);
        serde_json::to_value(schema).unwrap_or_default()
    }

    /// Deserialize JSON string with schema validation
    pub fn from_json_with_validation(
        json_str: &str,
    ) -> Result<Self, crate::domain::error::AppError> {
        // Use json5 for parsing and deserialization
        // This allows serde's alias feature to work, correctly parsing old snake_case keys
        let config: Self = json5::from_str(json_str).map_err(|e| {
            crate::domain::error::AppError::Validation(format!("Failed to parse config: {}", e))
        })?;

        // Convert normalized config to JSON Value for validation (all will be camelCase)
        let normalized_value = serde_json::to_value(&config).map_err(|e| {
            crate::domain::error::AppError::Validation(format!(
                "Failed to serialize normalized config: {}",
                e
            ))
        })?;

        // Schema validation
        let schema = Self::generate_schema();
        let compiled_schema = jsonschema::JSONSchema::compile(&schema).map_err(|e| {
            crate::domain::error::AppError::Validation(format!("Failed to compile schema: {}", e))
        })?;

        if let Err(errors) = compiled_schema.validate(&normalized_value) {
            let error_messages: Vec<String> = errors
                .map(|error| format!("Validation error at {}: {}", error.instance_path, error))
                .collect();
            return Err(crate::domain::error::AppError::Validation(format!(
                "Schema validation failed: {}",
                error_messages.join(", ")
            )));
        }

        Ok(config)
    }
}

/// Generate Commands JSON schema
pub fn generate_commands_schema() -> serde_json::Value {
    // Vec<Command> のスキーマを生成
    let schema = schema_for!(Vec<Command>);
    serde_json::to_value(schema).unwrap_or_default()
}

/// Deserialize JSON string with schema validation
pub fn commands_from_json_with_validation(
    json_str: &str,
) -> Result<Commands, crate::domain::error::AppError> {
    // Parse and deserialize JSON string
    // This allows serde's alias to work, reading snake_case keys correctly
    let commands: Commands = json5::from_str(json_str).map_err(|e| {
        crate::domain::error::AppError::Validation(format!("Failed to parse commands: {}", e))
    })?;

    // Parse normalized object for validation
    let normalized_value = serde_json::to_value(&commands).map_err(|e| {
        crate::domain::error::AppError::Validation(format!(
            "Failed to serialize normalized commands: {}",
            e
        ))
    })?;

    // Schema validation
    let schema = generate_commands_schema();
    let compiled_schema = jsonschema::JSONSchema::compile(&schema).map_err(|e| {
        crate::domain::error::AppError::Validation(format!("Failed to compile schema: {}", e))
    })?;

    if let Err(errors) = compiled_schema.validate(&normalized_value) {
        let error_messages: Vec<String> = errors
            .map(|error| format!("Validation error at {}: {}", error.instance_path, error))
            .collect();
        return Err(crate::domain::error::AppError::Validation(format!(
            "Schema validation failed: {}",
            error_messages.join(", ")
        )));
    }

    Ok(commands)
}

impl Config {
    /// Validate and fix config values
    pub fn validate_and_fix(&mut self) -> Result<(), crate::domain::error::AppError> {
        // fuzzy_threshold (0.0 - 1.0)
        if self.fuzzy_threshold < 0.0 || self.fuzzy_threshold > 1.0 {
            log::warn!(
                "Invalid fuzzy_threshold: {}, using default 0.5",
                self.fuzzy_threshold
            );
            self.fuzzy_threshold = 0.5;
        }
        Ok(())
    }
}
