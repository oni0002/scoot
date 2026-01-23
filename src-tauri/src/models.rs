use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

pub const CATEGORY_URL: &str = "url";
pub const CATEGORY_FILE: &str = "file";
pub const CATEGORY_BOOKMARK: &str = "bookmark";
pub const CATEGORY_COMMAND: &str = "command";
pub const CATEGORY_CUSTOM: &str = "custom";
pub const CATEGORY_SCOOT: &str = "scoot";
pub const CATEGORY_APPLICATION: &str = "application";
pub const DEFAULT_THEME: &str = "dark";
pub const DEFAULT_SHORTCUT: &str = "alt+space";

/// コマンド構造体
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub category: String,
    pub command: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(default = "default_editable")]
    pub is_editable: bool,
}

fn default_editable() -> bool {
    true
}

/// コマンドリスト
pub type Commands = Vec<Command>;

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
        }
    }
}

/// Commandのスキーマを生成
impl Command {
    /// カテゴリが有効かどうかを判定
    pub fn is_valid_category(&self) -> bool {
        return matches!(
            self.category.as_str(),
            CATEGORY_URL
                | CATEGORY_FILE
                | CATEGORY_BOOKMARK
                | CATEGORY_COMMAND
                | CATEGORY_CUSTOM
                | CATEGORY_SCOOT
                | CATEGORY_APPLICATION
        );
    }

    /// プレースホルダーがあるかチェック
    pub fn has_placeholders(&self) -> bool {
        self.command.contains('{') && self.command.contains('}')
    }

    /// 引数を置換してコマンドを生成
    pub fn substitute_args(&self, args: &[String]) -> String {
        let mut result = self.command.clone();

        // {$*} - 全引数をスペース区切りで結合
        if result.contains("{$*}") {
            let all_args = args.join(" ");
            result = result.replace("{$*}", &all_args);
        }

        // {$1}, {$2}, ... - 指定位置の引数（1ベース）
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{${}}}", i + 1);
            result = result.replace(&placeholder, arg);
        }

        result
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
        // JSONパース
        let json_value: serde_json::Value =
            serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

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
    // JSONパース
    let json_value: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_command(cmd: &str) -> Command {
        Command {
            id: "test".to_string(),
            name: "Test".to_string(),
            category: "general".to_string(),
            command: cmd.to_string(),
            description: "Test command".to_string(),
            prompt: None,
            working_dir: None,
            is_editable: true,
        }
    }

    #[test]
    fn test_substitute_args_all() {
        let cmd = create_dummy_command("echo {$*}");
        let args = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(cmd.substitute_args(&args), "echo hello world");
    }

    #[test]
    fn test_substitute_args_positional() {
        let cmd = create_dummy_command("mv {$1} {$2}");
        let args = vec!["src.txt".to_string(), "dest.txt".to_string()];
        assert_eq!(cmd.substitute_args(&args), "mv src.txt dest.txt");
    }

    #[test]
    fn test_substitute_args_no_placeholder() {
        let cmd = create_dummy_command("ls -la");
        let args = vec!["ignore".to_string()];
        assert_eq!(cmd.substitute_args(&args), "ls -la");
    }
}
