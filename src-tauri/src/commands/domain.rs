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
pub const SOURCE_SCOOT: &str = "scoot";

// Scoot command constants
pub const CMD_SCOOT_ADD_COMMAND: &str = "scoot://add-command";
pub const CMD_SCOOT_OPEN_COMMANDS: &str = "scoot://open-commands";
pub const CMD_SCOOT_OPEN_CONFIG: &str = "scoot://open-config";
pub const CMD_SCOOT_OPEN_SETTINGS: &str = "scoot://open-settings";
pub const CMD_SCOOT_OPEN_LOG: &str = "scoot://open-log";
pub const CMD_SCOOT_RELOAD: &str = "scoot://reload";
pub const CMD_SCOOT_KILL: &str = "scoot://kill";
pub const SCOOT_COMMANDS: [&str; 7] = [
    CMD_SCOOT_ADD_COMMAND,
    CMD_SCOOT_OPEN_COMMANDS,
    CMD_SCOOT_OPEN_CONFIG,
    CMD_SCOOT_OPEN_SETTINGS,
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
}

/// Deserialize commands.json. The fields' own rules are checked per command by `Command::validate`.
pub fn deserialize_json(json_str: &str) -> Result<Vec<Command>, crate::error::AppError> {
    crate::json::parse(json_str)
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
            "Open Config.json",
            CATEGORY_SCOOT,
            SOURCE_SCOOT,
            CMD_SCOOT_OPEN_CONFIG,
            "Open config.json configuration file",
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

        // Category length limit
        let category_length = self.category.chars().count();
        if category_length > 50 {
            return Err(crate::error::AppError::Validation(format!(
                "Command category is too long ({} characters). Maximum allowed is 50 characters.",
                category_length
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
        assert!(deserialize_json(json).is_ok());
    }
}
