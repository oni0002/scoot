use crate::commands::domain::Command;
use std::collections::HashMap;

pub struct CommandRegistry {
    next_id: usize,
    pub user_commands: HashMap<String, Command>,
    pub bookmark_commands: HashMap<String, Command>,
    pub scoot_commands: HashMap<String, Command>,
    pub application_commands: HashMap<String, Command>,
    pub markdown_commands: HashMap<String, Command>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            user_commands: HashMap::new(),
            bookmark_commands: HashMap::new(),
            scoot_commands: HashMap::new(),
            application_commands: HashMap::new(),
            markdown_commands: HashMap::new(),
        }
    }

    fn assign_id(&mut self, command: &mut Command) {
        self.next_id += 1;
        command.id = self.next_id.to_string();
    }

    pub fn set_scoot_commands(&mut self, commands: Vec<Command>) {
        self.scoot_commands.clear();
        for mut command in commands {
            self.assign_id(&mut command);
            self.scoot_commands.insert(command.id.clone(), command);
        }
    }

    pub fn add_user_command(&mut self, mut command: Command) -> String {
        self.assign_id(&mut command);
        let id = command.id.clone();
        self.user_commands.insert(id.clone(), command);
        id
    }

    pub fn update_user_command(&mut self, command: Command) -> Result<(), crate::error::AppError> {
        if !self.user_commands.contains_key(&command.id) {
            return Err(crate::error::AppError::NotFound(
                "Command not found".to_string(),
            ));
        }
        self.user_commands.insert(command.id.clone(), command);
        Ok(())
    }

    pub fn set_user_commands(&mut self, commands: Vec<Command>) {
        self.user_commands.clear();
        for command in commands {
            if self.validate_command(&command).is_ok() {
                self.add_user_command(command);
            } else {
                log::warn!(
                    "Skipping command '{}' ({}): Validation failed",
                    command.name,
                    command.id
                );
            }
        }
    }

    pub fn delete_user_command(&mut self, id: &str) -> Result<(), crate::error::AppError> {
        if self.user_commands.remove(id).is_none() {
            return Err(crate::error::AppError::NotFound(
                "Command not found".to_string(),
            ));
        }
        Ok(())
    }

    pub fn get_user_commands(&self) -> Vec<Command> {
        self.user_commands.values().cloned().collect()
    }

    pub fn get_commands_by_prompt(&self, prompt: &str) -> Vec<Command> {
        self.user_commands
            .values()
            .filter(|cmd| cmd.prompt.as_ref().map_or(false, |p| p == prompt))
            .cloned()
            .collect()
    }

    pub fn set_application_commands(&mut self, commands: Vec<Command>) {
        self.application_commands.clear();
        for mut command in commands {
            self.assign_id(&mut command);
            self.application_commands
                .insert(command.id.clone(), command);
        }
    }

    pub fn add_bookmark_command(&mut self, mut command: Command) {
        self.assign_id(&mut command);
        self.bookmark_commands.insert(command.id.clone(), command);
    }

    pub fn set_bookmark_commands(&mut self, commands: Vec<Command>) {
        self.bookmark_commands.clear();
        for command in commands {
            self.add_bookmark_command(command);
        }
    }

    pub fn clear_bookmark_commands(&mut self) {
        self.bookmark_commands.clear();
    }

    pub fn set_markdown_commands(&mut self, commands: Vec<Command>) {
        self.markdown_commands.clear();
        for mut command in commands {
            self.assign_id(&mut command);
            self.markdown_commands.insert(command.id.clone(), command);
        }
    }

    pub fn validate_command(&self, command: &Command) -> Result<(), crate::error::AppError> {
        command.validate()?;

        for existing_cmd in self.user_commands.values() {
            if existing_cmd.id != command.id && existing_cmd.command == command.command {
                return Err(crate::error::AppError::Validation(format!(
                    "This command path/URL is already registered as '{}'.",
                    existing_cmd.name
                )));
            }
        }

        if let Some(ref prompt) = command.prompt {
            if self.is_prompt_used(prompt, Some(&command.id)) {
                return Err(crate::error::AppError::Validation(format!(
                    "Prompt '{}' is already used by another command.",
                    prompt
                )));
            }
        }

        Ok(())
    }

    pub fn is_prompt_used(&self, prompt: &str, exclude_id: Option<&str>) -> bool {
        self.user_commands.values().any(|cmd| {
            if let Some(exclude) = exclude_id {
                if cmd.id == exclude {
                    return false;
                }
            }
            cmd.prompt.as_ref().map_or(false, |p| p == prompt)
        })
    }

    pub fn get_all_commands(&self) -> Vec<Command> {
        self.user_commands
            .values()
            .chain(self.bookmark_commands.values())
            .chain(self.scoot_commands.values())
            .chain(self.application_commands.values())
            .chain(self.markdown_commands.values())
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_command(prompt: Option<&str>) -> Command {
        Command {
            id: String::new(),
            name: "Test Command".to_string(),
            category: "command".to_string(),
            source: "user".to_string(),
            command: format!("echo test {}", prompt.unwrap_or("")),
            description: "Test".to_string(),
            prompt: prompt.map(|s| s.to_string()),
            working_dir: None,
            show_window: None,
        }
    }

    #[test]
    fn test_duplicate_prompt_check() {
        let mut manager = CommandRegistry::new();

        let cmd_a = create_dummy_command(Some("p1"));
        assert!(manager.validate_command(&cmd_a).is_ok());
        let id_a = manager.add_user_command(cmd_a);

        let mut cmd_b_dup = create_dummy_command(Some("p1"));
        cmd_b_dup.command = "echo cmd_b_dup".to_string();
        let res = manager.validate_command(&cmd_b_dup);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("already used"));

        let cmd_b = create_dummy_command(Some("p2"));
        assert!(manager.validate_command(&cmd_b).is_ok());
        let _id_b = manager.add_user_command(cmd_b);
        let mut cmd_a_update = manager.user_commands.get(&id_a).unwrap().clone();
        cmd_a_update.prompt = Some("p2".to_string());
        let res = manager.validate_command(&cmd_a_update);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("already used"));

        let mut cmd_a_same = manager.user_commands.get(&id_a).unwrap().clone();
        cmd_a_same.description = "Updated".to_string();
        assert!(manager.validate_command(&cmd_a_same).is_ok());
    }

    #[test]
    fn test_set_scoot_commands() {
        let mut manager = CommandRegistry::new();
        let scoot_cmd = create_dummy_command(None);

        manager.set_scoot_commands(vec![scoot_cmd]);

        let all = manager.get_all_commands();
        assert_eq!(all.len(), 1);
        assert!(!all[0].id.is_empty());
    }

    #[test]
    fn test_add_user_command_id_generation() {
        let mut manager = CommandRegistry::new();
        let cmd = create_dummy_command(None);

        let id = manager.add_user_command(cmd);
        assert!(!id.is_empty());
        assert!(manager.get_user_commands().iter().any(|c| c.id == id));
    }

    #[test]
    fn test_duplicate_command_check() {
        let mut manager = CommandRegistry::new();

        let mut cmd1 = create_dummy_command(Some("p1"));
        cmd1.command = "https://google.com".to_string();
        assert!(manager.validate_command(&cmd1).is_ok());
        manager.add_user_command(cmd1.clone());

        let mut cmd2 = create_dummy_command(Some("p2"));
        cmd2.command = "https://google.com".to_string();
        let res = manager.validate_command(&cmd2);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("already registered"));
    }
}
