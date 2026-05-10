use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
pub const CMD_SCOOT_OPEN_README: &str = "scoot://open-readme";
pub const CMD_SCOOT_OPEN_LOG: &str = "scoot://open-log";
pub const CMD_SCOOT_RELOAD: &str = "scoot://reload";
pub const CMD_SCOOT_KILL: &str = "scoot://kill";

/// Command struct
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_window: Option<bool>,
}

/// Generate Commands JSON schema
pub fn generate_commands_schema() -> serde_json::Value {
    let schema = schemars::schema_for!(Vec<Command>);
    serde_json::to_value(schema).unwrap_or_default()
}

/// Deserialize JSON string with schema validation
pub fn deserialize_json(json_str: &str) -> Result<Vec<Command>, crate::error::AppError> {
    crate::validation::parse_and_validate::<Vec<Command>>(json_str)
}

/// Get the built-in commands for Scoot
pub fn get_scoot_commands() -> Vec<Command> {
    vec![
        Command::new("Add Command", CATEGORY_SCOOT, SOURCE_SCOOT, CMD_SCOOT_ADD_COMMAND, "Add a new command to the launcher"),
        Command::new("Open Commands.json", CATEGORY_SCOOT, SOURCE_SCOOT, CMD_SCOOT_OPEN_COMMANDS, "Open commands.json configuration file"),
        Command::new("Open Config.json", CATEGORY_SCOOT, SOURCE_SCOOT, CMD_SCOOT_OPEN_CONFIG, "Open config.json configuration file"),
        Command::new("Open README", CATEGORY_SCOOT, SOURCE_SCOOT, CMD_SCOOT_OPEN_README, "Open application README"),
        Command::new("Open Logs", CATEGORY_SCOOT, SOURCE_SCOOT, CMD_SCOOT_OPEN_LOG, "Open application log directory"),
        Command::new("Reload", CATEGORY_SCOOT, SOURCE_SCOOT, CMD_SCOOT_RELOAD, "Reload commands and configuration"),
        Command::new("Kill Scoot", CATEGORY_SCOOT, SOURCE_SCOOT, CMD_SCOOT_KILL, "Terminate the application"),
    ]
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
            prompt: None,
            working_dir: None,
            show_window: None,
        }
    }

    /// Check if the command has placeholders
    pub fn has_placeholders(&self) -> bool {
        self.command.contains('{') && self.command.contains('}')
    }

    /// Substitute placeholders with arguments
    pub fn substitute_args(&self, args: &[String]) -> String {
        let mut result = self.command.clone();

        // {$*} - Substitute all arguments with a space-separated string
        if result.contains("{$*}") {
            let all_args = args.join(" ");
            result = result.replace("{$*}", &all_args);
        }

        // {$1}, {$2}, ... - Substitute arguments at specified positions (1-based)
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{${}}}", i + 1);
            result = result.replace(&placeholder, arg);
        }

        result
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
        if self.name.len() > 100 {
            return Err(crate::error::AppError::Validation(format!(
                "Command name is too long ({} characters). Maximum allowed is 100 characters.",
                self.name.len()
            )));
        }

        // Category length limit
        if self.category.len() > 50 {
            return Err(crate::error::AppError::Validation(format!(
                "Command category is too long ({} characters). Maximum allowed is 50 characters.",
                self.category.len()
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
        if self.command.len() > 1000 {
            return Err(crate::error::AppError::Validation(format!(
                "Command content is too long ({} characters). Maximum allowed is 1000 characters.",
                self.command.len()
            )));
        }

        // Description length limit
        if self.description.len() > 500 {
            return Err(crate::error::AppError::Validation(format!("Command description is too long ({} characters). Maximum allowed is 500 characters.", self.description.len())));
        }

        // Prompt validation
        if let Some(ref prompt) = self.prompt {
            if prompt.trim().is_empty() {
                return Err(crate::error::AppError::Validation("Prompt cannot be empty if specified. Either provide a valid prompt or leave it blank.".to_string()));
            }
            if prompt.len() > 10 {
                return Err(crate::error::AppError::Validation(format!(
                    "Prompt is too long ({} characters). Maximum allowed is 10 characters.",
                    prompt.len()
                )));
            }
            // Prompt contains whitespace characters (spaces, tabs, or newlines). Use a single word without spaces.
            if prompt.contains(' ') || prompt.contains('\t') || prompt.contains('\n') {
                return Err(crate::error::AppError::Validation("Prompt cannot contain whitespace characters (spaces, tabs, or newlines). Use a single word without spaces.".to_string()));
            }

            // Special characters check
            if prompt
                .chars()
                .any(|c| !c.is_alphanumeric() && c != '-' && c != '_')
            {
                return Err(crate::error::AppError::Validation(
                    "Prompt can only contain letters, numbers, hyphens (-), and underscores (_)."
                        .to_string(),
                ));
            }
        }

        // Scoot command validation
        if self.command.starts_with("scoot://") {
            let valid_scoot_commands = [
                CMD_SCOOT_ADD_COMMAND,
                CMD_SCOOT_OPEN_COMMANDS,
                CMD_SCOOT_OPEN_CONFIG,
                CMD_SCOOT_OPEN_README,
                CMD_SCOOT_OPEN_LOG,
                CMD_SCOOT_RELOAD,
                CMD_SCOOT_KILL,
            ];
            if !valid_scoot_commands.contains(&self.command.as_str()) {
                return Err(crate::error::AppError::Validation(format!(
                    "Invalid scoot command '{}'. Valid commands are: {}",
                    self.command,
                    valid_scoot_commands.join(", ")
                )));
            }
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
            prompt: None,
            working_dir: None,
            show_window: None,
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

    #[test]
    fn test_schema_with_skipped_id() {
        let schema = generate_commands_schema();
        let compiled = jsonschema::JSONSchema::compile(&schema).unwrap();

        let json =
            r#"[{"name": "test", "category": "url", "command": "http", "description": "desc"}]"#;
        let cmds: Vec<Command> = json5::from_str(json).unwrap();

        let normalized = serde_json::to_value(&cmds).unwrap();

        let result = compiled.validate(&normalized);
        if let Err(e) = result {
            for err in e {
                println!("Error: {}", err);
            }
            panic!("Validation failed");
        }
    }
}
