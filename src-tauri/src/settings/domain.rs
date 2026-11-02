use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
/// One day. Keep in sync with the input's `max` in src/components/settings/GeneralSection.tsx.
const MAX_RELOAD_INTERVAL_MINUTES: u64 = 24 * 60;
/// src/components/settings/LocalPathsSection.tsx の入力欄の `max` と揃える
const MAX_LOCAL_PATH_DEPTH: usize = 10;

/// Settings struct
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub max_results: usize,
    pub fuzzy_threshold: f64,
    pub bookmarks: BookmarkSettings,
    pub applications: ApplicationSettings,
    pub markdown: MarkdownSettings,
    pub local_paths: LocalPathSettings,
    pub ignored: Vec<String>,
    /// Aliases for external commands, keyed by their `command` string (like `ignored`). Their IDs
    /// are hashes that std does not promise to keep stable, so they cannot be persisted.
    pub aliases: BTreeMap<String, String>,
    pub theme: String,
    pub hotkey: String,
    pub reload_interval_minutes: u64,
    /// Whether runs are recorded and used to order search results
    pub usage_history: bool,
    /// Whether the most used commands are listed before anything is typed
    pub frequent_first: bool,
}

/// Bookmark settings struct
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BookmarkSettings {
    pub enabled: bool,
    pub browser: String,
}

/// Application settings struct
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApplicationSettings {
    pub enabled: bool,
    pub directories: Vec<String>,
    pub extensions: Vec<String>,
}

/// ApplicationSettings default values
impl Default for ApplicationSettings {
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

/// Markdown settings struct
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MarkdownSettings {
    pub enabled: bool,
    pub paths: Vec<String>,
}

/// MarkdownSettings default values
impl Default for MarkdownSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            paths: Vec::new(),
        }
    }
}

/// Local Path の設定。登録フォルダごとの一覧の仕方と、全フォルダに共通の除外名
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocalPathSettings {
    pub enabled: bool,
    pub entries: Vec<LocalPathEntry>,
    /// 中身ごと飛ばすファイル・フォルダの名前(大文字小文字は区別しない)
    pub exclude_names: Vec<String>,
}

impl Default for LocalPathSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            entries: Vec::new(),
            exclude_names: vec!["node_modules".to_string(), "target".to_string()],
        }
    }
}

/// 登録フォルダ1つと、その下のどこまで・何を一覧にするか
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocalPathEntry {
    /// `~` と `%VAR%` は読み込み時に展開する
    pub path: String,
    /// 登録フォルダから何階層下まで一覧にするか。1 はフォルダ直下だけ
    pub depth: usize,
    /// ドットなしの拡張子(大文字小文字は区別しない)。`*` はすべてのファイル、空ならファイルは出さない
    pub extensions: Vec<String>,
    pub include_folders: bool,
    /// `<alias> <keyword>` でこのフォルダの下だけを検索する。Command の Alias とは別の名前空間
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// 通常の検索には出さず、`alias` を打ったときだけ探せる(`alias` が要る)
    pub alias_only: bool,
}

impl Default for LocalPathEntry {
    fn default() -> Self {
        Self {
            path: String::new(),
            depth: 3,
            extensions: Vec::new(),
            include_folders: true,
            alias: None,
            alias_only: false,
        }
    }
}

/// BookmarkSettings default values
impl Default for BookmarkSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            browser: "brave".to_string(),
        }
    }
}

/// Settings default values
impl Default for Settings {
    fn default() -> Self {
        Self {
            max_results: 10,
            fuzzy_threshold: 0.5,
            bookmarks: BookmarkSettings::default(),
            applications: ApplicationSettings::default(),
            markdown: MarkdownSettings::default(),
            local_paths: LocalPathSettings::default(),
            ignored: Vec::new(),
            aliases: BTreeMap::new(),
            theme: DEFAULT_THEME.to_string(),
            hotkey: DEFAULT_SHORTCUT.to_string(),
            reload_interval_minutes: 30,
            usage_history: true,
            frequent_first: true,
        }
    }
}

impl Settings {
    /// An unknown `theme` (one of the old daisyUI names, say) falls back to the default instead of
    /// failing the whole settings: the theme is cosmetic, and the next save writes a valid value.
    pub fn from_json_with_validation(json_str: &str) -> Result<Self, crate::error::AppError> {
        let mut value: serde_json::Value = crate::json::parse(json_str)?;
        if let Some(theme) = value.get_mut("theme") {
            if theme.as_str().is_some_and(|t| !THEMES.contains(&t)) {
                log::warn!("Unknown theme {}; using '{}'.", theme, DEFAULT_THEME);
                *theme = serde_json::Value::from(DEFAULT_THEME);
            }
        }
        let settings: Self = serde_json::from_value(value).map_err(|e| {
            crate::error::AppError::Validation(format!("Failed to parse JSON: {}", e))
        })?;
        settings.validate()?;
        Ok(settings)
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
        if !(1..=MAX_RELOAD_INTERVAL_MINUTES).contains(&self.reload_interval_minutes) {
            return fail(format!(
                "reloadIntervalMinutes must be between 1 and {} (got {}).",
                MAX_RELOAD_INTERVAL_MINUTES, self.reload_interval_minutes
            ));
        }
        for (i, entry) in self.local_paths.entries.iter().enumerate() {
            if entry.path.trim().is_empty() {
                return fail(format!("localPaths.entries[{}].path must not be empty.", i));
            }
            if !(1..=MAX_LOCAL_PATH_DEPTH).contains(&entry.depth) {
                return fail(format!(
                    "localPaths.entries[{}].depth must be between 1 and {} (got {}).",
                    i, MAX_LOCAL_PATH_DEPTH, entry.depth
                ));
            }
            match &entry.alias {
                Some(alias) => {
                    crate::commands::domain::validate_alias(alias)?;
                    let first = self
                        .local_paths
                        .entries
                        .iter()
                        .position(|e| e.alias.as_deref() == Some(alias.as_str()));
                    if first != Some(i) {
                        return fail(format!(
                            "localPaths.entries[{}].alias '{}' is already used by another path.",
                            i, alias
                        ));
                    }
                }
                None if entry.alias_only => {
                    return fail(format!(
                        "localPaths.entries[{}].aliasOnly needs an alias.",
                        i
                    ));
                }
                None => {}
            }
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
        for alias in self.aliases.values() {
            crate::commands::domain::validate_alias(alias)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Result<Settings, crate::error::AppError> {
        Settings::from_json_with_validation(json)
    }

    #[test]
    fn default_settings_are_valid() {
        let json = serde_json::to_string(&Settings::default()).unwrap();
        assert!(parse(&json).is_ok());
    }

    #[test]
    fn default_application_directories_use_environment_variables() {
        // settings.json travels with the exe, so a drive letter or user name must not be baked in
        let dirs = ApplicationSettings::default().directories;
        assert!(!dirs.is_empty());
        assert!(dirs.iter().all(|d| d.starts_with('%')));
    }

    #[test]
    fn empty_object_falls_back_to_defaults() {
        let settings = parse("{}").unwrap();
        assert_eq!(settings.max_results, Settings::default().max_results);
    }

    #[test]
    fn nested_sections_may_leave_fields_out() {
        let settings =
            parse(r#"{"bookmarks": {"enabled": false}, "markdown": {"enabled": true}}"#).unwrap();
        assert!(!settings.bookmarks.enabled);
        assert_eq!(settings.bookmarks.browser, "brave");
        assert!(settings.markdown.enabled);
        assert!(settings.markdown.paths.is_empty());
        assert!(parse(r#"{"applications": {"enabled": false}}"#)
            .unwrap()
            .applications
            .directories
            .iter()
            .all(|d| d.starts_with('%')));
    }

    #[test]
    fn usage_history_is_on_unless_turned_off() {
        assert!(parse("{}").unwrap().usage_history);
        assert!(!parse(r#"{"usageHistory": false}"#).unwrap().usage_history);
    }

    #[test]
    fn frequent_first_is_on_unless_turned_off() {
        assert!(parse("{}").unwrap().frequent_first);
        assert!(!parse(r#"{"frequentFirst": false}"#).unwrap().frequent_first);
    }

    #[test]
    fn max_results_must_be_within_range() {
        assert!(parse(r#"{"maxResults": 1}"#).is_ok());
        assert!(parse(r#"{"maxResults": 100}"#).is_ok());
        assert!(parse(r#"{"maxResults": 0}"#).is_err());
        assert!(parse(r#"{"maxResults": 101}"#).is_err());
    }

    #[test]
    fn reload_interval_must_be_within_a_day() {
        assert!(parse(r#"{"reloadIntervalMinutes": 1}"#).is_ok());
        assert!(parse(r#"{"reloadIntervalMinutes": 1440}"#).is_ok());
        assert!(parse(r#"{"reloadIntervalMinutes": 0}"#).is_err());
        assert!(parse(r#"{"reloadIntervalMinutes": 1441}"#).is_err());
        // Large enough to overflow when turned into seconds
        assert!(parse(r#"{"reloadIntervalMinutes": 18446744073709551615}"#).is_err());
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
            let settings = parse(&format!(r#"{{"theme": "{theme}"}}"#)).unwrap();
            assert_eq!(settings.theme, theme);
        }
    }

    #[test]
    fn unknown_theme_falls_back_to_the_default() {
        for theme in ["nord", "dim", "not-a-theme", "darkness"] {
            let settings = parse(&format!(r#"{{"theme": "{theme}"}}"#)).unwrap();
            assert_eq!(settings.theme, DEFAULT_THEME);
        }
    }

    #[test]
    fn aliases_default_to_none_and_must_be_valid() {
        assert!(parse("{}").unwrap().aliases.is_empty());
        let settings = parse(r#"{"aliases": {"C:\\Apps\\Code.lnk": "vscode"}}"#).unwrap();
        assert_eq!(settings.aliases["C:\\Apps\\Code.lnk"], "vscode");
        assert!(parse(r#"{"aliases": {"x": "has space"}}"#).is_err());
        assert!(parse(r#"{"aliases": {"x": ""}}"#).is_err());
    }

    #[test]
    fn local_paths_are_off_by_default_and_skip_the_usual_build_folders() {
        let local = parse("{}").unwrap().local_paths;
        assert!(!local.enabled);
        assert!(local.entries.is_empty());
        assert_eq!(local.exclude_names, ["node_modules", "target"]);

        let local = parse(r#"{"localPaths": {"enabled": true, "entries": [{"path": "~/Projects", "depth": 2, "extensions": ["md", "*"], "includeFolders": false}], "excludeNames": []}}"#)
            .unwrap()
            .local_paths;
        assert!(local.enabled);
        assert_eq!(
            local.entries,
            [LocalPathEntry {
                path: "~/Projects".to_string(),
                depth: 2,
                extensions: vec!["md".to_string(), "*".to_string()],
                include_folders: false,
                alias: None,
                alias_only: false,
            }]
        );
        assert!(local.exclude_names.is_empty());
    }

    #[test]
    fn a_local_path_entry_may_give_only_its_path() {
        let local = parse(r#"{"localPaths": {"entries": [{"path": "C:\\Docs"}]}}"#)
            .unwrap()
            .local_paths;
        assert_eq!(local.entries.len(), 1);
        assert_eq!(local.entries[0].path, r"C:\Docs");
        assert_eq!(local.entries[0].depth, 3);
        assert!(local.entries[0].extensions.is_empty());
        assert!(local.entries[0].include_folders);
    }

    #[test]
    fn the_old_single_rule_shape_still_loads_with_no_entries() {
        // 未リリースの旧形式(全フォルダ共通の paths/depth/extensions/includeFolders)。移行はせず、
        // 古いキーは読み飛ばして他の設定は生かす
        let settings = parse(r#"{"maxResults": 7, "localPaths": {"enabled": true, "paths": ["C:\\Projects"], "depth": 2, "extensions": ["md"], "includeFolders": false, "excludeNames": ["dist"]}}"#)
            .unwrap();
        assert_eq!(settings.max_results, 7);
        assert!(settings.local_paths.enabled);
        assert!(settings.local_paths.entries.is_empty());
        assert_eq!(settings.local_paths.exclude_names, ["dist"]);
    }

    #[test]
    fn local_path_entries_need_a_path_and_a_depth_within_range() {
        let entry = |json: &str| parse(&format!(r#"{{"localPaths": {{"entries": [{json}]}}}}"#));
        assert!(entry(r#"{"path": "C:\\a", "depth": 1}"#).is_ok());
        assert!(entry(r#"{"path": "C:\\a", "depth": 10}"#).is_ok());
        assert!(entry(r#"{"path": "C:\\a", "depth": 0}"#).is_err());
        assert!(entry(r#"{"path": "C:\\a", "depth": 11}"#).is_err());
        assert!(entry(r#"{"path": "C:\\a", "depth": -1}"#).is_err());
        assert!(entry(r#"{"path": " "}"#).is_err());
        assert!(entry(r#"{"depth": 2}"#).is_err());
    }

    #[test]
    fn a_local_path_alias_is_well_formed_unique_and_required_by_alias_only() {
        let entries = |json: &str| parse(&format!(r#"{{"localPaths": {{"entries": [{json}]}}}}"#));
        let path = |alias: &str| format!(r#"{{"path": "C:\\a", "alias": "{alias}"}}"#);
        assert!(entries(&path("pj")).is_ok());
        assert!(entries(&path("has space")).is_err());
        assert!(entries(&path("")).is_err());
        assert!(entries(r#"{"path": "C:\\a", "aliasOnly": true}"#).is_err());
        assert!(entries(r#"{"path": "C:\\a", "alias": "pj", "aliasOnly": true}"#).is_ok());
        assert!(entries(&format!("{},{}", path("pj"), path("pj"))).is_err());
        assert!(entries(&format!("{},{}", path("pj"), path("nt"))).is_ok());
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
