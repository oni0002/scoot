use super::command::{Command, Commands};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

pub const DEFAULT_THEME: &str = "dark";
pub const DEFAULT_SHORTCUT: &str = "alt+space";

/// 設定構造体
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    #[schemars(range(min = 1, max = 100))]
    pub max_results: usize,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub fuzzy_threshold: f64,
    pub bookmarks: BookmarkConfig,
    pub applications: ApplicationConfig,
    #[schemars(regex(
        pattern = r"^(light|dark|cupcake|bumblebee|emerald|corporate|synthwave|retro|cyberpunk|valentine|halloween|garden|forest|aqua|lofi|pastel|fantasy|wireframe|black|luxury|dracula|cmyk|autumn|business|acid|lemonade|night|coffee|winter|dim|nord|sunset)$"
    ))]
    pub theme: String,
    #[schemars(regex(
        pattern = r"^((Cmd|Command|Ctrl|Control|Alt|Shift|Super|Option)\+)+([A-Z0-9a-z]|Space|Enter|Tab|F[1-9]|F1[0-2])$"
    ))]
    pub hotkey: String,
}

/// ブックマーク設定構造体
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BookmarkConfig {
    pub enabled: bool,
    pub browser: String,
    pub prompt: Option<String>,
    #[schemars(range(min = 1))]
    pub refresh_interval_minutes: u64,
}

/// アプリケーション設定構造体
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApplicationConfig {
    pub enabled: bool,
    pub directories: Vec<String>,
    pub extensions: Vec<String>,
}

/// ApplicationConfigのデフォルト値
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

/// BookmarkConfigのデフォルト値
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

/// Configのデフォルト値
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

/// スキーマ検証機能
impl Config {
    /// ConfigのJSONスキーマを生成
    pub fn generate_schema() -> serde_json::Value {
        let schema = schema_for!(Config);
        serde_json::to_value(schema).unwrap_or_default()
    }

    /// JSON文字列をスキーマ検証してからデシリアライズ
    pub fn from_json_with_validation(json_str: &str) -> Result<Self, String> {
        // json5を使用してパース (コメントと末尾カンマを許容)
        let json_value: serde_json::Value =
            json5::from_str(json_str).map_err(|e| format!("Invalid JSON5: {}", e))?;

        // スキーマ検証
        let schema = Self::generate_schema();
        let compiled_schema = jsonschema::JSONSchema::compile(&schema)
            .map_err(|e| format!("Failed to compile schema: {}", e))?;

        if let Err(errors) = compiled_schema.validate(&json_value) {
            let error_messages: Vec<String> = errors
                .map(|error| format!("Validation error at {}: {}", error.instance_path, error))
                .collect();
            return Err(format!(
                "Schema validation failed: {}",
                error_messages.join(", ")
            ));
        }

        // デシリアライゼーション
        serde_json::from_value(json_value).map_err(|e| format!("Failed to deserialize: {}", e))
    }
}

/// Commandsのスキーマを生成
pub fn generate_commands_schema() -> serde_json::Value {
    // Vec<Command> のスキーマを生成
    let schema = schema_for!(Vec<Command>);
    serde_json::to_value(schema).unwrap_or_default()
}

/// JSON文字列をスキーマ検証してからデシリアライズ
pub fn commands_from_json_with_validation(json_str: &str) -> Result<Commands, String> {
    // json5を使用してパース (コメントと末尾カンマを許容)
    let json_value: serde_json::Value =
        json5::from_str(json_str).map_err(|e| format!("Invalid JSON5: {}", e))?;

    // スキーマ検証
    let schema = generate_commands_schema();
    let compiled_schema = jsonschema::JSONSchema::compile(&schema)
        .map_err(|e| format!("Failed to compile schema: {}", e))?;

    if let Err(errors) = compiled_schema.validate(&json_value) {
        let error_messages: Vec<String> = errors
            .map(|error| format!("Validation error at {}: {}", error.instance_path, error))
            .collect();
        return Err(format!(
            "Schema validation failed: {}",
            error_messages.join(", ")
        ));
    }

    // デシリアライゼーション
    serde_json::from_value(json_value).map_err(|e| format!("Failed to deserialize: {}", e))
}

impl Config {
    /// 設定の値を検証し、不正な値があれば修正する
    pub fn validate_and_fix(&mut self) -> Result<(), String> {
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
