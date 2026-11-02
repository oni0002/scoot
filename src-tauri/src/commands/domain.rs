use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// `{$1}`, `{$2}`, ... or `{$*}`. Keep in sync with `PLACEHOLDER_RE` in src/command.ts.
static PLACEHOLDER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\$([1-9][0-9]*|\*)\}").expect("invalid regex"));

// Category constants (execution type)
pub const CATEGORY_URL: &str = "url";
pub const CATEGORY_FILE: &str = "file";
pub const CATEGORY_COMMAND: &str = "command";
pub const CATEGORY_SCOOT: &str = "scoot";

// Source constants (data origin)
pub const SOURCE_USER: &str = "user";
pub const SOURCE_BOOKMARK: &str = "bookmark";
pub const SOURCE_APPLICATION: &str = "application";
pub const SOURCE_LOCAL: &str = "local";
pub const SOURCE_SCOOT: &str = "scoot";

/// Local Path の登録フォルダの Alias に入る行の `command` の頭。実行はせず、UI が `<alias> ` に入る。
/// Keep in sync with `SCOPE_COMMAND_PREFIX` in src/command.ts.
pub const SCOPE_COMMAND_PREFIX: &str = "scoot://search-folder/";

// Scoot command constants
pub const CMD_SCOOT_ADD_COMMAND: &str = "scoot://add-command";
pub const CMD_SCOOT_OPEN_COMMANDS: &str = "scoot://open-commands";
pub const CMD_SCOOT_OPEN_SETTINGS_JSON: &str = "scoot://open-settings-json";
pub const CMD_SCOOT_OPEN_SETTINGS: &str = "scoot://open-settings";
pub const CMD_SCOOT_OPEN_LOG: &str = "scoot://open-log";
pub const CMD_SCOOT_RELOAD: &str = "scoot://reload";
pub const CMD_SCOOT_KILL: &str = "scoot://kill";
pub const SCOOT_COMMANDS: [&str; 7] = [
    CMD_SCOOT_ADD_COMMAND,
    CMD_SCOOT_OPEN_COMMANDS,
    CMD_SCOOT_OPEN_SETTINGS,
    CMD_SCOOT_OPEN_SETTINGS_JSON,
    CMD_SCOOT_OPEN_LOG,
    CMD_SCOOT_RELOAD,
    CMD_SCOOT_KILL,
];

/// Command struct
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub name: String,
    pub category: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source: String,
    pub command: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "prompt")]
    pub alias: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_window: Option<bool>,
    /// command を UAC で昇格して実行する(Windows のみ)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_as_admin: Option<bool>,
    /// url / file を開くプログラム。未設定なら既定のアプリで開く
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "blank_as_none"
    )]
    pub open_with: Option<String>,
    /// open_with に対象より前に渡す固定の引数
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "blank_as_none"
    )]
    pub open_args: Option<String>,
    /// Local Path で、この項目を一覧にした登録フォルダの Alias。実行時のみ(外部ソースの項目にだけ付く)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
    /// 一覧にした登録フォルダがすべて alias_only で、通常の検索には出さない
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub scoped_only: bool,
}

/// 空白だけの文字列は未設定として扱う
fn blank_as_none<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<String>::deserialize(d)?.filter(|s| !s.trim().is_empty()))
}

/// Get the built-in commands for Scoot
pub fn get_scoot_commands() -> Vec<Command> {
    vec![
        Command::new(
            "Add Command",
            CATEGORY_SCOOT,
            SOURCE_SCOOT,
            CMD_SCOOT_ADD_COMMAND,
            "Add a new command to the launcher",
        ),
        Command::new(
            "Open Settings",
            CATEGORY_SCOOT,
            SOURCE_SCOOT,
            CMD_SCOOT_OPEN_SETTINGS,
            "Open the settings screen",
        ),
        Command::new(
            "Open Commands.json",
            CATEGORY_SCOOT,
            SOURCE_SCOOT,
            CMD_SCOOT_OPEN_COMMANDS,
            "Open commands.json configuration file",
        ),
        Command::new(
            "Open Settings.json",
            CATEGORY_SCOOT,
            SOURCE_SCOOT,
            CMD_SCOOT_OPEN_SETTINGS_JSON,
            "Open settings.json configuration file",
        ),
        Command::new(
            "Open Logs",
            CATEGORY_SCOOT,
            SOURCE_SCOOT,
            CMD_SCOOT_OPEN_LOG,
            "Open application log directory",
        ),
        Command::new(
            "Reload",
            CATEGORY_SCOOT,
            SOURCE_SCOOT,
            CMD_SCOOT_RELOAD,
            "Reload commands and configuration",
        ),
        Command::new(
            "Kill Scoot",
            CATEGORY_SCOOT,
            SOURCE_SCOOT,
            CMD_SCOOT_KILL,
            "Terminate the application",
        ),
    ]
}

/// Format rules for an alias, shared by user commands and the aliases set on external ones
pub fn validate_alias(alias: &str) -> Result<(), crate::error::AppError> {
    let fail = |msg: String| Err(crate::error::AppError::Validation(msg));
    if alias.trim().is_empty() {
        return fail(
            "Alias cannot be empty if specified. Either provide a valid alias or leave it blank."
                .to_string(),
        );
    }
    let length = alias.chars().count();
    if length > 10 {
        return fail(format!(
            "Alias is too long ({} characters). Maximum allowed is 10 characters.",
            length
        ));
    }
    if alias.contains(' ') || alias.contains('\t') || alias.contains('\n') {
        return fail("Alias cannot contain whitespace characters (spaces, tabs, or newlines). Use a single word without spaces.".to_string());
    }
    if alias
        .chars()
        .any(|c| !c.is_alphanumeric() && c != '-' && c != '_')
    {
        return fail(
            "Alias can only contain letters, numbers, hyphens (-), and underscores (_)."
                .to_string(),
        );
    }
    Ok(())
}

/// Percent-encode everything except RFC 3986 unreserved characters
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Command methods
impl Command {
    pub fn new(
        name: impl Into<String>,
        category: impl Into<String>,
        source: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Command {
            id: String::new(),
            name: name.into(),
            category: category.into(),
            source: source.into(),
            command: command.into(),
            description: description.into(),
            alias: None,
            working_dir: None,
            show_window: None,
            run_as_admin: None,
            open_with: None,
            open_args: None,
            scopes: Vec::new(),
            scoped_only: false,
        }
    }

    /// Check if the command has placeholders
    pub fn has_placeholders(&self) -> bool {
        PLACEHOLDER_RE.is_match(&self.command)
    }

    /// The placeholders that `args` leaves without a value, in order of appearance
    pub fn missing_slots(&self, args: &[String]) -> Vec<String> {
        let mut missing: Vec<String> = Vec::new();
        for caps in PLACEHOLDER_RE.captures_iter(&self.command) {
            let is_missing = match &caps[1] {
                "*" => args.is_empty(),
                n => n.parse::<usize>().map_or(true, |n| n > args.len()),
            };
            if is_missing && !missing.contains(&caps[0].to_string()) {
                missing.push(caps[0].to_string());
            }
        }
        missing
    }

    /// Substitute placeholders with arguments in a single pass, so a placeholder inside an
    /// argument is never expanded again. A placeholder without a value is left as it is.
    /// URL commands get percent-encoded arguments so that characters like `&` or `#`
    /// cannot alter the structure of the URL.
    /// Keep in sync with `expandTemplate` in src/command.ts; both run the cases in
    /// src/__tests__/fixtures/substitute-cases.json.
    pub fn substitute_args(&self, args: &[String]) -> String {
        let is_url = self.category == CATEGORY_URL;
        let values: Vec<String> = if is_url {
            args.iter().map(|a| percent_encode(a)).collect()
        } else {
            args.to_vec()
        };
        let all = values.join(if is_url { "%20" } else { " " });

        PLACEHOLDER_RE
            .replace_all(&self.command, |caps: &Captures| {
                let value = match &caps[1] {
                    "*" => (!values.is_empty()).then(|| all.clone()),
                    n => n
                        .parse::<usize>()
                        .ok()
                        .and_then(|n| values.get(n - 1))
                        .cloned(),
                };
                value.unwrap_or_else(|| caps[0].to_string())
            })
            .into_owned()
    }

    /// Validate command format
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        // Required fields validation
        if self.name.trim().is_empty() {
            return Err(crate::error::AppError::Validation(
                "Command name is required and cannot be empty or contain only whitespace."
                    .to_string(),
            ));
        }
        if self.command.trim().is_empty() {
            return Err(crate::error::AppError::Validation(
                "Command content is required and cannot be empty or contain only whitespace."
                    .to_string(),
            ));
        }
        if self.category.trim().is_empty() {
            return Err(crate::error::AppError::Validation(
                "Command category is required and cannot be empty or contain only whitespace."
                    .to_string(),
            ));
        }

        // Name length limit
        let name_length = self.name.chars().count();
        if name_length > 100 {
            return Err(crate::error::AppError::Validation(format!(
                "Command name is too long ({} characters). Maximum allowed is 100 characters.",
                name_length
            )));
        }

        // Category validity check
        if !matches!(
            self.category.as_str(),
            CATEGORY_URL | CATEGORY_FILE | CATEGORY_COMMAND | CATEGORY_SCOOT
        ) {
            return Err(crate::error::AppError::Validation(format!(
                "Invalid category '{}'. Supported categories are: url, file, command, scoot.",
                self.category
            )));
        }

        // Command content length limit
        let command_length = self.command.chars().count();
        if command_length > 1000 {
            return Err(crate::error::AppError::Validation(format!(
                "Command content is too long ({} characters). Maximum allowed is 1000 characters.",
                command_length
            )));
        }

        // Description length limit
        let description_length = self.description.chars().count();
        if description_length > 500 {
            return Err(crate::error::AppError::Validation(format!("Command description is too long ({} characters). Maximum allowed is 500 characters.", description_length)));
        }

        match &self.alias {
            Some(alias) => validate_alias(alias)?,
            // Only an alias gets the arguments typed after it to the command
            None if self.has_placeholders() => {
                return Err(crate::error::AppError::Validation(
                    "A command with placeholders such as {$1} needs an alias to receive its arguments."
                        .to_string(),
                ));
            }
            None => {}
        }

        if (self.open_with.is_some() || self.open_args.is_some())
            && !matches!(self.category.as_str(), CATEGORY_URL | CATEGORY_FILE)
        {
            return Err(crate::error::AppError::Validation(
                "Open With is only available for url and file commands.".to_string(),
            ));
        }

        if self.open_args.is_some() && self.open_with.is_none() {
            return Err(crate::error::AppError::Validation(
                "Open With is required when open args are set.".to_string(),
            ));
        }

        if self.run_as_admin.is_some() && self.category != CATEGORY_COMMAND {
            return Err(crate::error::AppError::Validation(
                "Run as Administrator is only available for command commands.".to_string(),
            ));
        }

        // Scoot command validation
        if self.command.starts_with("scoot://") && !SCOOT_COMMANDS.contains(&self.command.as_str())
        {
            return Err(crate::error::AppError::Validation(format!(
                "Invalid scoot command '{}'. Valid commands are: {}",
                self.command,
                SCOOT_COMMANDS.join(", ")
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_command(cmd: &str) -> Command {
        Command {
            id: "test".to_string(),
            name: "Test".to_string(),
            category: "command".to_string(),
            source: "user".to_string(),
            command: cmd.to_string(),
            description: "Test command".to_string(),
            alias: None,
            working_dir: None,
            show_window: None,
            run_as_admin: None,
            open_with: None,
            open_args: None,
            scopes: Vec::new(),
            scoped_only: false,
        }
    }

    #[derive(Deserialize)]
    struct SubstituteCase {
        name: String,
        category: String,
        command: String,
        args: Vec<String>,
        expected: String,
    }

    // The same cases src/__tests__/command.test.ts runs against `substituteArgs`
    #[test]
    fn substitute_args_matches_the_shared_cases() {
        let cases: Vec<SubstituteCase> = serde_json::from_str(include_str!(
            "../../../src/__tests__/fixtures/substitute-cases.json"
        ))
        .unwrap();
        assert!(!cases.is_empty());
        for case in cases {
            let mut cmd = create_dummy_command(&case.command);
            cmd.category = case.category;
            assert_eq!(
                cmd.substitute_args(&case.args),
                case.expected,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn missing_slots_lists_placeholders_without_a_value() {
        let args = |list: &[&str]| list.iter().map(|a| a.to_string()).collect::<Vec<_>>();
        let cmd = create_dummy_command("cmd {$1} {$2} {$2}");
        assert_eq!(cmd.missing_slots(&args(&["a"])), ["{$2}"]);
        assert!(cmd.missing_slots(&args(&["a", "b"])).is_empty());
        let all = create_dummy_command("cmd {$*}");
        assert_eq!(all.missing_slots(&[]), ["{$*}"]);
        assert!(all.missing_slots(&args(&["a"])).is_empty());
        assert!(create_dummy_command("cmd {$0} {$01}")
            .missing_slots(&[])
            .is_empty());
    }

    #[test]
    fn only_the_known_categories_are_accepted() {
        for category in ["url", "file", "command", "scoot"] {
            let mut cmd = create_dummy_command("echo hi");
            cmd.category = category.to_string();
            assert!(cmd.validate().is_ok(), "{}", category);
        }
        for category in ["", "URL", "shell", &"x".repeat(80)] {
            let mut cmd = create_dummy_command("echo hi");
            cmd.category = category.to_string();
            assert!(cmd.validate().is_err(), "{}", category);
        }
    }

    #[test]
    fn open_with_is_only_for_url_and_file() {
        for (category, ok) in [
            ("url", true),
            ("file", true),
            ("command", false),
            ("scoot", false),
        ] {
            let mut cmd = create_dummy_command("echo hi");
            cmd.category = category.to_string();
            cmd.open_with = Some("chrome.exe".to_string());
            assert_eq!(cmd.validate().is_ok(), ok, "{}", category);
            cmd.open_args = Some("--incognito".to_string());
            assert_eq!(cmd.validate().is_ok(), ok, "{}", category);
            cmd.open_with = None;
            assert!(cmd.validate().is_err(), "{}", category);
        }
    }

    #[test]
    fn open_args_need_open_with() {
        let mut cmd = create_dummy_command("https://a.b");
        cmd.category = "url".to_string();
        cmd.open_args = Some("--incognito".to_string());
        let err = cmd.validate().unwrap_err().to_string();
        assert!(err.contains("Open With"), "{}", err);
        cmd.open_with = Some("chrome.exe".to_string());
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn run_as_admin_is_only_for_command() {
        for (category, ok) in [
            ("command", true),
            ("url", false),
            ("file", false),
            ("scoot", false),
        ] {
            let mut cmd = create_dummy_command("echo hi");
            cmd.category = category.to_string();
            cmd.run_as_admin = Some(true);
            assert_eq!(cmd.validate().is_ok(), ok, "{}", category);
        }
        let mut cmd = create_dummy_command("echo hi");
        cmd.run_as_admin = Some(true);
        assert_eq!(serde_json::to_value(&cmd).unwrap()["runAsAdmin"], true);
    }

    #[test]
    fn open_with_fields_serialize_camel_case_and_blank_means_unset() {
        let json = r#"{"name":"n","category":"url","command":"https://a.b","description":"",
            "openWith":"chrome.exe","openArgs":"--incognito"}"#;
        let cmd: Command = crate::json::parse(json).unwrap();
        let out = serde_json::to_value(&cmd).unwrap();
        assert_eq!(out["openWith"], "chrome.exe");
        assert_eq!(out["openArgs"], "--incognito");

        let blank = r#"{"name":"n","category":"url","command":"https://a.b","description":"",
            "openWith":"  ","openArgs":""}"#;
        let cmd: Command = crate::json::parse(blank).unwrap();
        assert!(cmd.open_with.is_none() && cmd.open_args.is_none());
        let out = serde_json::to_value(&cmd).unwrap();
        assert!(out.get("openWith").is_none() && out.get("openArgs").is_none());
    }

    #[test]
    fn placeholders_need_an_alias() {
        let mut cmd = create_dummy_command("echo {$1}");
        assert!(cmd.validate().is_err());
        cmd.alias = Some("e".to_string());
        assert!(cmd.validate().is_ok());
        assert!(create_dummy_command("echo hi").validate().is_ok());
    }

    #[test]
    fn lengths_count_characters_not_bytes() {
        let mut cmd = create_dummy_command("echo hi");
        cmd.alias = Some("あいうえおかきくけこ".to_string()); // 10 characters, 30 bytes
        assert!(cmd.validate().is_ok());
        cmd.alias = Some("あいうえおかきくけこさ".to_string());
        assert!(cmd.validate().is_err());
        cmd.alias = None;
        cmd.name = "あ".repeat(100);
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_missing_id_is_accepted() {
        let json =
            r#"[{"name": "test", "category": "url", "command": "http", "description": "desc"}]"#;
        assert!(crate::json::parse::<Vec<Command>>(json).is_ok());
    }
}
