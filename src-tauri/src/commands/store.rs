use crate::commands::domain::Command;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global ID counter for generating unique command IDs
static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Generate a unique ID
fn generate_id() -> String {
    ID_COUNTER.fetch_add(1, Ordering::Relaxed).to_string()
}

/// CommandManager struct
pub struct CommandManager {
    // User defined commands
    pub user_commands: HashMap<String, Command>,
    // Bookmark commands
    pub bookmark_commands: HashMap<String, Command>,
    // Scoot commands
    pub scoot_commands: HashMap<String, Command>,
    // Application commands
    pub application_commands: HashMap<String, Command>,
}

/// CommandManager implementation
impl CommandManager {
    pub fn new() -> Self {
        Self {
            user_commands: HashMap::new(),
            bookmark_commands: HashMap::new(),
            scoot_commands: HashMap::new(),
            application_commands: HashMap::new(),
        }
    }

    /// Assign a new ID to a command
    fn assign_id(command: &mut Command) {
        command.id = generate_id();
    }

    /// Set Scoot commands
    pub fn set_scoot_commands(&mut self, commands: Vec<Command>) {
        self.scoot_commands.clear();
        for mut command in commands {
            Self::assign_id(&mut command);
            self.scoot_commands.insert(command.id.clone(), command);
        }
    }

    /// Add user command
    pub fn add_user_command(&mut self, mut command: Command) -> String {
        Self::assign_id(&mut command);
        let id = command.id.clone();
        self.user_commands.insert(id.clone(), command);
        id
    }

    /// Update user command
    pub fn update_user_command(
        &mut self,
        command: Command,
    ) -> Result<(), crate::error::AppError> {
        if !self.user_commands.contains_key(&command.id) {
            return Err(crate::error::AppError::NotFound(
                "Command not found".to_string(),
            ));
        }

        self.user_commands.insert(command.id.clone(), command);
        Ok(())
    }

    /// Delete user command
    pub fn delete_user_command(&mut self, id: &str) -> Result<(), crate::error::AppError> {
        if self.user_commands.remove(id).is_none() {
            return Err(crate::error::AppError::NotFound(
                "Command not found".to_string(),
            ));
        }
        Ok(())
    }

    /// Get command by ID
    #[allow(dead_code)]
    pub fn get_command(&self, id: &str) -> Option<&Command> {
        self.user_commands
            .get(id)
            .or_else(|| self.bookmark_commands.get(id))
            .or_else(|| self.scoot_commands.get(id))
            .or_else(|| self.application_commands.get(id))
    }

    /// Get all commands
    pub fn get_all_commands(&self) -> Vec<Command> {
        self.user_commands
            .values()
            .chain(self.bookmark_commands.values())
            .chain(self.scoot_commands.values())
            .chain(self.application_commands.values())
            .cloned()
            .collect()
    }

    /// Get user commands only (for commands.json)
    pub fn get_user_commands(&self) -> Vec<Command> {
        self.user_commands.values().cloned().collect()
    }

    /// Add bookmark command
    pub fn add_bookmark_command(&mut self, mut command: Command) {
        Self::assign_id(&mut command);
        self.bookmark_commands.insert(command.id.clone(), command);
    }

    /// Clear bookmarks
    pub fn clear_bookmarks(&mut self) {
        self.bookmark_commands.clear();
    }

    /// Get commands by category
    #[allow(dead_code)]
    pub fn get_commands_by_category(&self, category: &str) -> Vec<Command> {
        self.user_commands
            .values()
            .filter(|cmd| cmd.category == category)
            .cloned()
            .collect()
    }

    /// Get commands by prompt
    pub fn get_commands_by_prompt(&self, prompt: &str) -> Vec<Command> {
        self.user_commands
            .values()
            .filter(|cmd| cmd.prompt.as_ref().map_or(false, |p| p == prompt))
            .cloned()
            .collect()
    }

    /// Validate command
    pub fn validate_command(
        &self,
        command: &Command,
    ) -> Result<(), crate::error::AppError> {
        // Domain level validation (format etc.)
        command.validate()?;

        // Store level validation (prompt uniqueness)
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

    /// Prompt duplicate check
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

    /// Get categories
    #[allow(dead_code)]
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .user_commands
            .values()
            .map(|cmd| cmd.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    /// Get command count by category
    #[allow(dead_code)]
    pub fn count_by_category(&self, category: &str) -> usize {
        self.user_commands
            .values()
            .filter(|cmd| cmd.category == category)
            .count()
    }

    /// Clear user commands
    pub fn clear_user_commands(&mut self) {
        self.user_commands.clear();
    }

    /// Set application commands
    pub fn set_application_commands(&mut self, commands: Vec<Command>) {
        self.application_commands.clear();
        for mut command in commands {
            Self::assign_id(&mut command);
            self.application_commands
                .insert(command.id.clone(), command);
        }
    }

    /// Set bookmark commands
    pub fn set_bookmark_commands(&mut self, commands: Vec<Command>) {
        self.bookmark_commands.clear();
        for mut command in commands {
            Self::assign_id(&mut command);
            self.bookmark_commands.insert(command.id.clone(), command);
        }
    }

    /// Set user commands (with validation)
    pub fn set_user_commands(&mut self, commands: Vec<Command>) {
        self.user_commands.clear();
        for command in commands {
            // Category validation
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_command(prompt: Option<&str>) -> Command {
        Command {
            id: String::new(),
            name: "Test Command".to_string(),
            category: "command".to_string(),
            command: "echo test".to_string(),
            description: "Test".to_string(),
            prompt: prompt.map(|s| s.to_string()),
            working_dir: None,
            show_window: None,
        }
    }

    #[test]
    fn test_duplicate_prompt_check() {
        let mut manager = CommandManager::new();

        // 1. 繧ｳ繝槭Φ繝陰繧定ｿｽ蜉 (prompt: p1)
        let cmd_a = create_dummy_command(Some("p1"));
        assert!(manager.validate_command(&cmd_a).is_ok());
        let id_a = manager.add_user_command(cmd_a);

        // 2. 繧ｳ繝槭Φ繝隠繧定ｿｽ蜉 (prompt: p1) -> 驥崎､・お繝ｩ繝ｼ
        let cmd_b_dup = create_dummy_command(Some("p1"));
        let res = manager.validate_command(&cmd_b_dup);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("already used"));

        // 3. 繧ｳ繝槭Φ繝隠繧定ｿｽ蜉 (prompt: p2) -> 謌仙粥
        let cmd_b = create_dummy_command(Some("p2"));
        assert!(manager.validate_command(&cmd_b).is_ok());
        let _id_b = manager.add_user_command(cmd_b);

        // 4. 繧ｳ繝槭Φ繝陰繧呈峩譁ｰ (prompt: p2) -> 驥崎､・お繝ｩ繝ｼ
        let mut cmd_a_update = manager.user_commands.get(&id_a).unwrap().clone();
        cmd_a_update.prompt = Some("p2".to_string());
        let res = manager.validate_command(&cmd_a_update);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("already used"));

        // 5. 繧ｳ繝槭Φ繝陰繧呈峩譁ｰ (prompt: p1) -> 謌仙粥 (閾ｪ蛻・・霄ｫ)
        let mut cmd_a_same = manager.user_commands.get(&id_a).unwrap().clone();
        cmd_a_same.description = "Updated".to_string();
        assert!(manager.validate_command(&cmd_a_same).is_ok());
    }

    #[test]
    fn test_set_scoot_commands() {
        let mut manager = CommandManager::new();
        let scoot_cmd = create_dummy_command(None);

        manager.set_scoot_commands(vec![scoot_cmd]);

        let all = manager.get_all_commands();
        assert_eq!(all.len(), 1);
        assert!(!all[0].id.is_empty()); // ID縺瑚・蜍慕函謌舌＆繧後※縺・ｋ縺薙→
    }

    #[test]
    fn test_add_user_command_id_generation() {
        let mut manager = CommandManager::new();
        let cmd = create_dummy_command(None);

        let id = manager.add_user_command(cmd);
        assert!(!id.is_empty());
        assert!(manager.get_user_commands().iter().any(|c| c.id == id));
    }
}

