use crate::commands::domain::Command;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub struct CommandRegistry {
    pub user_commands: HashMap<String, Command>,
    external_commands: Vec<Command>, // bookmark + app + markdown + scoot (reload-only)
    external_index: HashMap<String, usize>, // id → position in external_commands
    command_index: HashMap<String, String>, // command string → id
    alias_index: HashMap<String, String>, // alias string → id
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            user_commands: HashMap::new(),
            external_commands: Vec::new(),
            external_index: HashMap::new(),
            command_index: HashMap::new(),
            alias_index: HashMap::new(),
        }
    }

    // Counter-based IDs restart every launch and could collide with persisted user IDs
    fn assign_id(command: &mut Command) {
        if command.id.is_empty() {
            command.id = nanoid::nanoid!();
        }
    }

    /// A reload must hand out the same ID for the same command, or an ID the webview already holds
    /// would stop resolving. Derived from what identifies the command, not from a counter.
    fn external_id(command: &Command) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (&command.source, &command.category, &command.command).hash(&mut hasher);
        format!("ext-{:016x}", hasher.finish())
    }

    pub fn set_external_commands(&mut self, commands: Vec<Command>) {
        let mut seen = HashSet::new();
        self.external_commands = commands
            .into_iter()
            .map(|mut command| {
                let mut id = Self::external_id(&command);
                // Two commands that identify alike still need distinct IDs
                while !seen.insert(id.clone()) {
                    id.push('x');
                }
                command.id = id;
                command
            })
            .collect();
        self.rebuild_external_index();
    }

    /// Drop external commands by their command string (used when one is ignored)
    pub fn remove_external_by_command(&mut self, command: &str) {
        self.external_commands.retain(|c| c.command != command);
        self.rebuild_external_index();
    }

    fn rebuild_external_index(&mut self) {
        self.external_index = self
            .external_commands
            .iter()
            .enumerate()
            .map(|(i, c)| (c.id.clone(), i))
            .collect();
    }

    pub fn add_user_command(&mut self, mut command: Command) -> String {
        Self::assign_id(&mut command);
        let id = command.id.clone();
        self.command_index
            .insert(command.command.clone(), id.clone());
        if let Some(ref a) = command.alias {
            self.alias_index.insert(a.clone(), id.clone());
        }
        self.user_commands.insert(id.clone(), command);
        id
    }

    pub fn update_user_command(&mut self, command: Command) -> Result<(), crate::error::AppError> {
        let old = self
            .user_commands
            .get(&command.id)
            .ok_or_else(|| crate::error::AppError::NotFound("Command not found".to_string()))?;
        // Remove old index entries
        self.command_index.remove(&old.command);
        if let Some(ref a) = old.alias {
            self.alias_index.remove(a);
        }
        // Insert new index entries
        self.command_index
            .insert(command.command.clone(), command.id.clone());
        if let Some(ref a) = command.alias {
            self.alias_index.insert(a.clone(), command.id.clone());
        }
        self.user_commands.insert(command.id.clone(), command);
        Ok(())
    }

    pub fn set_user_commands(&mut self, commands: Vec<Command>) {
        self.user_commands.clear();
        self.command_index.clear();
        self.alias_index.clear();
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
        let command = self
            .user_commands
            .remove(id)
            .ok_or_else(|| crate::error::AppError::NotFound("Command not found".to_string()))?;
        self.command_index.remove(&command.command);
        if let Some(ref a) = command.alias {
            self.alias_index.remove(a);
        }
        Ok(())
    }

    pub fn get_user_commands(&self) -> Vec<Command> {
        self.user_commands.values().cloned().collect()
    }

    pub fn validate_command(&self, command: &Command) -> Result<(), crate::error::AppError> {
        command.validate()?;

        if let Some(existing_id) = self.command_index.get(&command.command) {
            if existing_id != &command.id {
                let name = self
                    .user_commands
                    .get(existing_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("unknown");
                return Err(crate::error::AppError::Validation(format!(
                    "This command path/URL is already registered as '{}'.",
                    name
                )));
            }
        }

        if let Some(ref alias) = command.alias {
            if self.is_alias_used(alias, Some(&command.id)) {
                return Err(crate::error::AppError::Validation(format!(
                    "Alias '{}' is already used by another command.",
                    alias
                )));
            }
        }

        Ok(())
    }

    pub fn is_alias_used(&self, alias: &str, exclude_id: Option<&str>) -> bool {
        match self.alias_index.get(alias) {
            Some(id) => exclude_id.is_none_or(|ex| ex != id),
            None => false,
        }
    }

    pub fn get_command(&self, id: &str) -> Option<Command> {
        self.user_commands
            .get(id)
            .or_else(|| {
                self.external_index
                    .get(id)
                    .and_then(|&i| self.external_commands.get(i))
            })
            .cloned()
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

    fn create_dummy_command(alias: Option<&str>) -> Command {
        Command {
            id: String::new(),
            name: "Test Command".to_string(),
            category: "command".to_string(),
            source: "user".to_string(),
            command: format!("echo test {}", alias.unwrap_or("")),
            description: "Test".to_string(),
            alias: alias.map(|s| s.to_string()),
            working_dir: None,
            show_window: None,
        }
    }

    #[test]
    fn test_duplicate_alias_check() {
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
        cmd_a_update.alias = Some("p2".to_string());
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
    fn test_get_command_finds_user_and_external() {
        let mut manager = CommandRegistry::new();
        let user_id = manager.add_user_command(create_dummy_command(Some("p1")));
        manager.set_external_commands(vec![create_dummy_command(None)]);
        let external_id = manager.get_all_commands().last().unwrap().id.clone();

        assert!(manager.get_command(&user_id).is_some());
        assert!(manager.get_command(&external_id).is_some());
        assert!(manager.get_command("missing").is_none());
    }

    #[test]
    fn external_ids_survive_a_reload() {
        let mut manager = CommandRegistry::new();
        manager.set_external_commands(vec![create_dummy_command(None)]);
        let before = manager.get_all_commands()[0].id.clone();

        manager.set_external_commands(vec![
            create_dummy_command(Some("x")),
            create_dummy_command(None),
        ]);
        assert!(manager.get_command(&before).is_some());
    }

    #[test]
    fn external_ids_stay_distinct_for_identical_commands() {
        let mut manager = CommandRegistry::new();
        manager.set_external_commands(vec![create_dummy_command(None), create_dummy_command(None)]);
        let all = manager.get_all_commands();
        assert_ne!(all[0].id, all[1].id);
    }

    #[test]
    fn removing_an_external_command_keeps_lookups_correct() {
        let mut manager = CommandRegistry::new();
        manager.set_external_commands(vec![
            create_dummy_command(Some("a")),
            create_dummy_command(Some("b")),
        ]);
        let kept = manager.get_all_commands()[1].clone();
        manager.remove_external_by_command(&manager.get_all_commands()[0].command.clone());
        assert_eq!(manager.get_command(&kept.id).unwrap().command, kept.command);
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
