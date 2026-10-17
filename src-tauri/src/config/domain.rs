use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static HOTKEY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^((Cmd|Command|Ctrl|Control|Alt|Shift|Super|Option|cmd|command|ctrl|control|alt|shift|super|option)\+)+([A-Z0-9a-z]|Space|Enter|Tab|F[1-9]|F1[0-2]|space|enter|tab|f[1-9]|f1[0-2])$",
    )
    .expect("invalid regex")
});

pub const DEFAULT_THEME: &str = "system";
const THEMES: [&str; 3] = ["system", "light", "dark"];
pub const DEFAULT_SHORTCUT: &str = "Alt+Space";

/// Config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub max_results: usize,
    pub fuzzy_threshold: f64,
    pub bookmarks: BookmarkConfig,
    pub applications: ApplicationConfig,
    pub markdown: MarkdownConfig,
    pub ignored: Vec<String>,
    pub theme: String,
    pub hotkey: String,
    pub reload_interval_minutes: u64,
    /// Whether runs are recorded and used to order search results
    pub usage_history: bool,
}

/// Bookmark config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkConfig {
    pub enabled: bool,
    pub browser: String,
}

/// Application config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
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
                r"%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs".to_string(),
            ],
            extensions: vec!["lnk".to_string()],
        }
    }
}

/// Markdown config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            usage_history: true,
        }
    }
}

impl Config {
    /// An unknown `theme` (one of the old daisyUI names, say) falls back to the default instead of
    /// failing the whole config: the theme is cosmetic, and the next save writes a valid value.
    pub fn from_json_with_validation(json_str: &str) -> Result<Self, crate::error::AppError> {
        let mut value: serde_json::Value = crate::json::parse(json_str)?;
        if let Some(theme) = value.get_mut("theme") {
            if theme.as_str().is_some_and(|t| !THEMES.contains(&t)) {
                log::warn!("Unknown theme {}; using '{}'.", theme, DEFAULT_THEME);
                *theme = serde_json::Value::from(DEFAULT_THEME);
            }
        }
        let config: Self = serde_json::from_value(value).map_err(|e| {
            crate::error::AppError::Validation(format!("Failed to parse JSON: {}", e))
        })?;
        config.validate()?;
        Ok(config)
    }

    /// Range and format rules for the values serde cannot check by type alone
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        let fail = |msg: String| Err(crate::error::AppError::Validation(msg));
        if !(1..=100).contains(&self.max_results) {
            return fail(format!(
                "maxResults must be between 1 and 100 (got {}).",
                self.max_results
            ));
        }
        if !(0.0..=1.0).contains(&self.fuzzy_threshold) {
            return fail(format!(
                "fuzzyThreshold must be between 0 and 1 (got {}).",
                self.fuzzy_threshold
            ));
        }
        if !THEMES.contains(&self.theme.as_str()) {
            return fail(format!(
                "theme must be one of {} (got '{}').",
                THEMES.join(", "),
                self.theme
            ));
        }
        if !HOTKEY_RE.is_match(&self.hotkey) {
            return fail(format!(
                "hotkey '{}' is not a valid shortcut such as Alt+Space or Ctrl+Shift+K.",
                self.hotkey
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Result<Config, crate::error::AppError> {
        Config::from_json_with_validation(json)
    }

    #[test]
    fn default_config_is_valid() {
        let json = serde_json::to_string(&Config::default()).unwrap();
        assert!(parse(&json).is_ok());
    }

    #[test]
    fn default_application_directories_use_environment_variables() {
        // config.json travels with the exe, so a drive letter or user name must not be baked in
        let dirs = ApplicationConfig::default().directories;
        assert!(!dirs.is_empty());
        assert!(dirs.iter().all(|d| d.starts_with('%')));
    }

    #[test]
    fn empty_object_falls_back_to_defaults() {
        let config = parse("{}").unwrap();
        assert_eq!(config.max_results, Config::default().max_results);
    }

    #[test]
    fn usage_history_is_on_unless_turned_off() {
        assert!(parse("{}").unwrap().usage_history);
        assert!(!parse(r#"{"usageHistory": false}"#).unwrap().usage_history);
    }

    #[test]
    fn max_results_must_be_within_range() {
        assert!(parse(r#"{"maxResults": 1}"#).is_ok());
        assert!(parse(r#"{"maxResults": 100}"#).is_ok());
        assert!(parse(r#"{"maxResults": 0}"#).is_err());
        assert!(parse(r#"{"maxResults": 101}"#).is_err());
    }

    #[test]
    fn fuzzy_threshold_must_be_within_range() {
        assert!(parse(r#"{"fuzzyThreshold": 0.0}"#).is_ok());
        assert!(parse(r#"{"fuzzyThreshold": 1.0}"#).is_ok());
        assert!(parse(r#"{"fuzzyThreshold": -0.1}"#).is_err());
        assert!(parse(r#"{"fuzzyThreshold": 1.1}"#).is_err());
    }

    #[test]
    fn theme_accepts_system_light_and_dark() {
        for theme in ["system", "light", "dark"] {
            let config = parse(&format!(r#"{{"theme": "{theme}"}}"#)).unwrap();
            assert_eq!(config.theme, theme);
        }
    }

    #[test]
    fn unknown_theme_falls_back_to_the_default() {
        for theme in ["nord", "dim", "not-a-theme", "darkness"] {
            let config = parse(&format!(r#"{{"theme": "{theme}"}}"#)).unwrap();
            assert_eq!(config.theme, DEFAULT_THEME);
        }
    }

    #[test]
    fn hotkey_must_match_the_pattern() {
        assert!(parse(r#"{"hotkey": "Ctrl+Shift+K"}"#).is_ok());
        assert!(parse(r#"{"hotkey": "Alt+Space"}"#).is_ok());
        assert!(parse(r#"{"hotkey": "Space"}"#).is_err());
        assert!(parse(r#"{"hotkey": "Alt+"}"#).is_err());
        assert!(parse(r#"{"hotkey": "garbage"}"#).is_err());
    }
}
