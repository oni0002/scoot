use crate::commands::domain::Command;
use std::collections::HashMap;

pub struct CommandRegistry {
    next_id: usize,
    pub user_commands: HashMap<String, Command>,
    pub external_commands: Vec<Command>, // bookmark + app + markdown + scoot (reload-only)
    command_index: HashMap<String, String>, // command string → id
    prompt_index: HashMap<String, String>,  // prompt string → id
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            user_commands: HashMap::new(),
            external_commands: Vec::new(),
            command_index: HashMap::new(),
            prompt_index: HashMap::new(),
        }
    }

    fn assign_id(&mut self, command: &mut Command) {
        if command.id.is_empty() {
            self.next_id += 1;
            command.id = self.next_id.to_string();
        }
    }

    pub fn set_external_commands(&mut self, commands: Vec<Command>) {
        self.external_commands.clear();
        for mut command in commands {
            self.assign_id(&mut command);
            self.external_commands.push(command);
        }
    }

    pub fn add_user_command(&mut self, mut command: Command) -> String {
        self.assign_id(&mut command);
        let id = command.id.clone();
        self.command_index.insert(command.command.clone(), id.clone());
        if let Some(ref p) = command.prompt {
            self.prompt_index.insert(p.clone(), id.clone());
        }
        self.user_commands.insert(id.clone(), command);
        id
    }

    pub fn update_user_command(&mut self, command: Command) -> Result<(), crate::error::AppError> {
        let old = self.user_commands.get(&command.id).ok_or_else(|| {
            crate::error::AppError::NotFound("Command not found".to_string())
        })?;
        // Remove old index entries
        self.command_index.remove(&old.command);
        if let Some(ref p) = old.prompt {
            self.prompt_index.remove(p);
        }
        // Insert new index entries
        self.command_index.insert(command.command.clone(), command.id.clone());
        if let Some(ref p) = command.prompt {
            self.prompt_index.insert(p.clone(), command.id.clone());
        }
        self.user_commands.insert(command.id.clone(), command);
        Ok(())
    }

    pub fn set_user_commands(&mut self, commands: Vec<Command>) {
        self.user_commands.clear();
        self.command_index.clear();
        self.prompt_index.clear();
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
        let command = self.user_commands.remove(id).ok_or_else(|| {
            crate::error::AppError::NotFound("Command not found".to_string())
        })?;
        self.command_index.remove(&command.command);
        if let Some(ref p) = command.prompt {
            self.prompt_index.remove(p);
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

    pub fn validate_command(&self, command: &Command) -> Result<(), crate::error::AppError> {
        command.validate()?;

        if let Some(existing_id) = self.command_index.get(&command.command) {
            if existing_id != &command.id {
                let name = self.user_commands.get(existing_id).map(|c| c.name.as_str()).unwrap_or("unknown");
                return Err(crate::error::AppError::Validation(format!(
                    "This command path/URL is already registered as '{}'.",
                    name
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
        match self.prompt_index.get(prompt) {
            Some(id) => exclude_id.map_or(true, |ex| ex != id),
            None => false,
        }
    }

    pub fn get_all_commands(&self) -> Vec<Command> {
        self.user_commands
            .values()
            .chain(self.external_commands.iter())
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
    fn test_set_external_commands() {
        let mut manager = CommandRegistry::new();
        let cmd = create_dummy_command(None);

        manager.set_external_commands(vec![cmd]);

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
