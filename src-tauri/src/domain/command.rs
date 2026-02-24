use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CATEGORY_URL: &str = "url";
pub const CATEGORY_FILE: &str = "file";
pub const CATEGORY_BOOKMARK: &str = "bookmark";
pub const CATEGORY_COMMAND: &str = "command";
pub const CATEGORY_SCOOT: &str = "scoot";
pub const CATEGORY_APPLICATION: &str = "application";

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
    pub id: String,
    pub name: String,
    pub category: String,
    pub command: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        // TODO: Remove `alias = "working_dir"` in v1.0.0 (Legacy config support)
        alias = "working_dir"
    )]
    pub working_dir: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        // TODO: Remove `alias = "show_window"` in v1.0.0 (Legacy config support)
        alias = "show_window"
    )]
    pub show_window: Option<bool>,
    // TODO: Remove `alias = "is_editable"` in v1.0.0 (Legacy config support)
    #[serde(default = "default_editable", alias = "is_editable")]
    pub is_editable: bool,
}

fn default_editable() -> bool {
    true
}

/// Command list
pub type Commands = Vec<Command>;

/// Command methods
impl Command {
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
    pub fn validate(&self) -> Result<(), crate::domain::error::AppError> {
        // Required fields validation
        if self.name.trim().is_empty() {
            return Err(crate::domain::error::AppError::Validation(
                "Command name is required and cannot be empty or contain only whitespace."
                    .to_string(),
            ));
        }
        if self.command.trim().is_empty() {
            return Err(crate::domain::error::AppError::Validation(
                "Command content is required and cannot be empty or contain only whitespace."
                    .to_string(),
            ));
        }
        if self.category.trim().is_empty() {
            return Err(crate::domain::error::AppError::Validation(
                "Command category is required and cannot be empty or contain only whitespace."
                    .to_string(),
            ));
        }

        // Name length limit
        if self.name.len() > 100 {
            return Err(crate::domain::error::AppError::Validation(format!(
                "Command name is too long ({} characters). Maximum allowed is 100 characters.",
                self.name.len()
            )));
        }

        // Category length limit
        if self.category.len() > 50 {
            return Err(crate::domain::error::AppError::Validation(format!(
                "Command category is too long ({} characters). Maximum allowed is 50 characters.",
                self.category.len()
            )));
        }

        // Category validity check
        if !matches!(
            self.category.as_str(),
            CATEGORY_URL
                | CATEGORY_FILE
                | CATEGORY_BOOKMARK
                | CATEGORY_COMMAND
                | CATEGORY_SCOOT
                | CATEGORY_APPLICATION
        ) {
            return Err(crate::domain::error::AppError::Validation(format!(
                "Invalid category '{}'. Supported categories are: url, file, bookmark, command, scoot, application.",
                self.category
            )));
        }

        // Command content length limit
        if self.command.len() > 1000 {
            return Err(crate::domain::error::AppError::Validation(format!(
                "Command content is too long ({} characters). Maximum allowed is 1000 characters.",
                self.command.len()
            )));
        }

        // Description length limit
        if self.description.len() > 500 {
            return Err(crate::domain::error::AppError::Validation(format!("Command description is too long ({} characters). Maximum allowed is 500 characters.", self.description.len())));
        }

        // Prompt validation
        if let Some(ref prompt) = self.prompt {
            if prompt.trim().is_empty() {
                return Err(crate::domain::error::AppError::Validation("Prompt cannot be empty if specified. Either provide a valid prompt or leave it blank.".to_string()));
            }
            if prompt.len() > 10 {
                return Err(crate::domain::error::AppError::Validation(format!(
                    "Prompt is too long ({} characters). Maximum allowed is 10 characters.",
                    prompt.len()
                )));
            }
            // Prompt contains whitespace characters (spaces, tabs, or newlines). Use a single word without spaces.
            if prompt.contains(' ') || prompt.contains('\t') || prompt.contains('\n') {
                return Err(crate::domain::error::AppError::Validation("Prompt cannot contain whitespace characters (spaces, tabs, or newlines). Use a single word without spaces.".to_string()));
            }

            // Special characters check
            if prompt
                .chars()
                .any(|c| !c.is_alphanumeric() && c != '-' && c != '_')
            {
                return Err(crate::domain::error::AppError::Validation(
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
                return Err(crate::domain::error::AppError::Validation(format!(
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
            category: "general".to_string(),
            command: cmd.to_string(),
            description: "Test command".to_string(),
            prompt: None,
            working_dir: None,
            show_window: None,
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
