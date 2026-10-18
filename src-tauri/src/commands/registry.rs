use crate::commands::domain::Command;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub struct CommandRegistry {
    user_commands: Vec<Command>, // in the order of commands.json
    /// Commands from commands.json that failed validation. They are neither offered nor run, but a
    /// save writes them back, so an unrelated edit never deletes what the user typed by hand.
    rejected_commands: Vec<Command>,
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
            user_commands: Vec::new(),
            rejected_commands: Vec::new(),
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
        self.user_commands.push(command);
        id
    }

    pub fn update_user_command(&mut self, command: Command) -> Result<(), crate::error::AppError> {
        let position = self
            .user_commands
            .iter()
            .position(|c| c.id == command.id)
            .ok_or_else(|| crate::error::AppError::NotFound("Command not found".to_string()))?;
        // Remove old index entries
        let old = &self.user_commands[position];
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
        self.user_commands[position] = command;
        Ok(())
    }

    pub fn set_user_commands(&mut self, commands: Vec<Command>) {
        self.user_commands.clear();
        self.rejected_commands.clear();
        self.command_index.clear();
        self.alias_index.clear();
        for command in commands {
            if let Err(e) = self.validate_command(&command) {
                log::warn!(
                    "Skipping command '{}' ({}): {}",
                    command.name,
                    command.id,
                    e
                );
                self.rejected_commands.push(command);
            } else {
                self.add_user_command(command);
            }
        }
    }

    pub fn delete_user_command(&mut self, id: &str) -> Result<(), crate::error::AppError> {
        let position = self
            .user_commands
            .iter()
            .position(|c| c.id == id)
            .ok_or_else(|| crate::error::AppError::NotFound("Command not found".to_string()))?;
        let command = self.user_commands.remove(position);
        self.command_index.remove(&command.command);
        if let Some(ref a) = command.alias {
            self.alias_index.remove(a);
        }
        Ok(())
    }

    /// What commands.json should hold: the valid commands, then the ones that failed validation
    pub fn get_user_commands(&self) -> Vec<Command> {
        self.user_commands
            .iter()
            .chain(&self.rejected_commands)
            .cloned()
            .collect()
    }

    pub fn validate_command(&self, command: &Command) -> Result<(), crate::error::AppError> {
        command.validate()?;

        if let Some(existing_id) = self.command_index.get(&command.command) {
            if existing_id != &command.id {
                let name = self
                    .user_commands
                    .iter()
                    .find(|c| &c.id == existing_id)
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

    /// An alias is taken if a user command or an external command already carries it. The
    /// command being given the alias is left out of the check, so it can keep what it has.
    fn alias_taken(
        &self,
        alias: &str,
        is_self_user: impl Fn(&str) -> bool,
        is_self_external: impl Fn(&Command) -> bool,
    ) -> bool {
        self.alias_index
            .get(alias)
            .is_some_and(|id| !is_self_user(id))
            || self
                .external_commands
                .iter()
                .any(|c| c.alias.as_deref() == Some(alias) && !is_self_external(c))
    }

    /// Whether `alias` is taken by any command but the user command `exclude_id`
    fn is_alias_used(&self, alias: &str, exclude_id: Option<&str>) -> bool {
        self.alias_taken(
            alias,
            |id| exclude_id == Some(id),
            |c| exclude_id == Some(c.id.as_str()),
        )
    }

    /// Set or clear the alias of the external commands whose `command` string is `command_str`
    /// (the same command can come from two sources, and both take it). The alias must be free
    /// everywhere else.
    pub fn set_external_alias(
        &mut self,
        command_str: &str,
        alias: Option<&str>,
    ) -> Result<(), crate::error::AppError> {
        if !self
            .external_commands
            .iter()
            .any(|c| c.command == command_str)
        {
            return Err(crate::error::AppError::NotFound(
                "Command not found".to_string(),
            ));
        }
        if let Some(alias) = alias {
            crate::commands::domain::validate_alias(alias)?;
            // The same command string may come from two sources; they take the alias together
            if self.alias_taken(alias, |_| false, |c| c.command == command_str) {
                return Err(crate::error::AppError::Validation(format!(
                    "Alias '{}' is already used by another command.",
                    alias
                )));
            }
        }
        for c in self
            .external_commands
            .iter_mut()
            .filter(|c| c.command == command_str)
        {
            c.alias = alias.map(str::to_string);
        }
        Ok(())
    }

    pub fn get_command(&self, id: &str) -> Option<Command> {
        self.user_commands
            .iter()
            .find(|c| c.id == id)
            .or_else(|| {
                self.external_index
                    .get(id)
                    .and_then(|&i| self.external_commands.get(i))
            })
            .cloned()
    }

    pub fn get_all_commands(&self) -> Vec<Command> {
        self.user_commands
            .iter()
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
        let mut cmd_a_update = manager.get_command(&id_a).unwrap();
        cmd_a_update.alias = Some("p2".to_string());
        let res = manager.validate_command(&cmd_a_update);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("already used"));

        let mut cmd_a_same = manager.get_command(&id_a).unwrap();
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
    fn external_alias_is_set_cleared_and_unique() {
        let mut manager = CommandRegistry::new();
        manager.add_user_command(create_dummy_command(Some("taken")));
        let mut a = create_dummy_command(None);
        a.command = "C:\\Apps\\a.lnk".to_string();
        let mut b = create_dummy_command(None);
        b.command = "C:\\Apps\\b.lnk".to_string();
        manager.set_external_commands(vec![a, b]);

        manager
            .set_external_alias("C:\\Apps\\a.lnk", Some("aa"))
            .unwrap();
        let alias_of = |m: &CommandRegistry, cmd: &str| {
            m.get_all_commands()
                .into_iter()
                .find(|c| c.command == cmd)
                .unwrap()
                .alias
        };
        assert_eq!(alias_of(&manager, "C:\\Apps\\a.lnk").as_deref(), Some("aa"));

        // Taken by a user command, by another external command, or malformed
        assert!(manager
            .set_external_alias("C:\\Apps\\b.lnk", Some("taken"))
            .is_err());
        assert!(manager
            .set_external_alias("C:\\Apps\\b.lnk", Some("aa"))
            .is_err());
        assert!(manager
            .set_external_alias("C:\\Apps\\b.lnk", Some("a a"))
            .is_err());
        assert!(manager.set_external_alias("missing", Some("zz")).is_err());

        // Setting it again on the same command is not a clash
        assert!(manager
            .set_external_alias("C:\\Apps\\a.lnk", Some("aa"))
            .is_ok());

        manager.set_external_alias("C:\\Apps\\a.lnk", None).unwrap();
        assert_eq!(alias_of(&manager, "C:\\Apps\\a.lnk"), None);
    }

    #[test]
    fn user_command_cannot_take_an_external_alias() {
        let mut manager = CommandRegistry::new();
        let mut ext = create_dummy_command(Some("code"));
        ext.source = "application".to_string();
        manager.set_external_commands(vec![ext]);

        let user = create_dummy_command(Some("code"));
        let res = manager.validate_command(&user);
        assert!(res.unwrap_err().to_string().contains("already used"));
    }

    #[test]
    fn user_commands_keep_their_order_through_edits() {
        let mut manager = CommandRegistry::new();
        let names = ["a", "b", "c", "d"];
        let ids: Vec<String> = names
            .iter()
            .map(|n| {
                let mut cmd = create_dummy_command(None);
                cmd.name = n.to_string();
                cmd.command = format!("echo {}", n);
                manager.add_user_command(cmd)
            })
            .collect();
        let order = |m: &CommandRegistry| -> Vec<String> {
            m.get_user_commands().into_iter().map(|c| c.name).collect()
        };

        let mut edited = manager.get_command(&ids[1]).unwrap();
        edited.description = "edited".to_string();
        manager.update_user_command(edited).unwrap();
        assert_eq!(order(&manager), ["a", "b", "c", "d"]);

        manager.delete_user_command(&ids[2]).unwrap();
        assert_eq!(order(&manager), ["a", "b", "d"]);
    }

    #[test]
    fn commands_that_fail_validation_survive_a_save() {
        let mut manager = CommandRegistry::new();
        let mut valid = create_dummy_command(None);
        valid.name = "valid".to_string();
        let mut invalid = create_dummy_command(None);
        invalid.name = "invalid".to_string();
        invalid.command = "echo other".to_string();
        invalid.category = "nonsense".to_string();
        manager.set_user_commands(vec![invalid, valid]);

        // Not offered or run ...
        let all: Vec<String> = manager
            .get_all_commands()
            .into_iter()
            .map(|c| c.name)
            .collect();
        assert_eq!(all, ["valid"]);
        // ... but still written back, after the valid ones
        let saved: Vec<String> = manager
            .get_user_commands()
            .into_iter()
            .map(|c| c.name)
            .collect();
        assert_eq!(saved, ["valid", "invalid"]);

        // A reload that no longer finds it forgets it
        manager.set_user_commands(Vec::new());
        assert!(manager.get_user_commands().is_empty());
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
