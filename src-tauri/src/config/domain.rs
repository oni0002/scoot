use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

pub const DEFAULT_THEME: &str = "dark";
pub const DEFAULT_SHORTCUT: &str = "Alt+Space";

/// Config struct
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    #[schemars(range(min = 1, max = 100))]
    pub max_results: usize,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub fuzzy_threshold: f64,
    pub bookmarks: BookmarkConfig,
    pub applications: ApplicationConfig,
    pub markdown: MarkdownConfig,
    pub ignored: Vec<String>,
    #[schemars(regex(
        pattern = r"^(light|dark|cupcake|bumblebee|emerald|corporate|synthwave|retro|cyberpunk|valentine|halloween|garden|forest|aqua|lofi|pastel|fantasy|wireframe|black|luxury|dracula|cmyk|autumn|business|acid|lemonade|night|coffee|winter|dim|nord|sunset)$"
    ))]
    pub theme: String,
    #[schemars(regex(
        pattern = r"^((Cmd|Command|Ctrl|Control|Alt|Shift|Super|Option|cmd|command|ctrl|control|alt|shift|super|option)\+)+([A-Z0-9a-z]|Space|Enter|Tab|F[1-9]|F1[0-2]|space|enter|tab|f[1-9]|f1[0-2])$"
    ))]
    pub hotkey: String,
    pub reload_interval_minutes: u64,
}

/// Bookmark config struct
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkConfig {
    pub enabled: bool,
    pub browser: String,
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

/// Markdown config struct
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownConfig {
    pub enabled: bool,
    pub paths: Vec<String>,
}

/// MarkdownConfig default values
impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            paths: Vec::new(),
        }
    }
}

/// BookmarkConfig default values
impl Default for BookmarkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            browser: "brave".to_string(),
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
            markdown: MarkdownConfig::default(),
            ignored: Vec::new(),
            theme: DEFAULT_THEME.to_string(),
            hotkey: DEFAULT_SHORTCUT.to_string(),
            reload_interval_minutes: 30,
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
    pub fn from_json_with_validation(json_str: &str) -> Result<Self, crate::error::AppError> {
        crate::validation::parse_and_validate::<Self>(json_str)
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::domain::{generate_commands_schema, Commands};

    #[test]
    fn test_schema_with_skipped_id() {
        let schema = generate_commands_schema();
        let compiled = jsonschema::JSONSchema::compile(&schema).unwrap();

        let json =
            r#"[{"name": "test", "category": "url", "command": "http", "description": "desc"}]"#;
        let cmds: Commands = json5::from_str(json).unwrap();

        // Convert to JSON Value to validate
        let normalized = serde_json::to_value(&cmds).unwrap();

        // This will panic if invalid
        let result = compiled.validate(&normalized);
        if let Err(e) = result {
            for err in e {
                println!("Error: {}", err);
            }
            panic!("Validation failed");
        }
    }
}
