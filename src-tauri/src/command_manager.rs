use crate::models::Command;
use std::collections::HashMap;
use uuid::Uuid;

/// CommandManagerの構造体
pub struct CommandManager {
    pub commands: HashMap<String, Command>,
    pub bookmark_commands: HashMap<String, Command>,
    pub scoot_commands: HashMap<String, Command>,
    pub application_commands: HashMap<String, Command>,
}

/// CommandManager
impl CommandManager {
    /// 新しいCommandManagerを生成
    pub fn new() -> Self {
        let mut manager = Self {
            commands: HashMap::new(),
            bookmark_commands: HashMap::new(),
            scoot_commands: HashMap::new(),
            application_commands: HashMap::new(),
        };
        manager.initialize_scoot_commands();
        manager
    }

    /// 新しいコマンドを追加
    pub fn add_command(&mut self, mut command: Command) -> String {
        // IDが空の場合は新しいUUIDを生成
        if command.id.is_empty() {
            command.id = Uuid::new_v4().to_string();
        }
        // ID重複チェック
        let mut final_id = command.id.clone();
        let mut counter = 1;
        while self.commands.contains_key(&final_id) {
            final_id = format!("{}-{}", command.id, counter);
            counter += 1;
        }
        command.id = final_id.clone();

        // コマンド一覧に追加
        self.commands.insert(final_id.clone(), command);
        final_id
    }

    /// コマンドを更新
    pub fn update_command(&mut self, command: Command) -> Result<(), String> {
        if !self.commands.contains_key(&command.id) {
            return Err("Command not found".to_string());
        }

        self.commands.insert(command.id.clone(), command);
        Ok(())
    }

    /// コマンドを削除
    pub fn delete_command(&mut self, id: &str) -> Result<(), String> {
        if self.commands.remove(id).is_none() {
            return Err("Command not found".to_string());
        }
        Ok(())
    }

    /// IDでコマンドを取得
    #[allow(dead_code)]
    pub fn get_command(&self, id: &str) -> Option<&Command> {
        self.commands
            .get(id)
            .or_else(|| self.bookmark_commands.get(id))
            .or_else(|| self.scoot_commands.get(id))
            .or_else(|| self.application_commands.get(id))
    }

    /// 全てのコマンドを取得
    pub fn get_all_commands(&self) -> Vec<Command> {
        let mut all_commands = Vec::new();
        // 通常コマンド
        all_commands.extend(self.commands.values().cloned());
        // ブックマークコマンド
        all_commands.extend(self.bookmark_commands.values().cloned());
        // Scootコマンド
        all_commands.extend(self.scoot_commands.values().cloned());
        // アプリケーションコマンド
        all_commands.extend(self.application_commands.values().cloned());
        all_commands
    }

    /// ユーザー定義コマンドのみ取得 (commands.json用)
    pub fn get_user_commands(&self) -> Vec<Command> {
        self.commands.values().cloned().collect()
    }

    /// ブックマークコマンドを追加
    pub fn add_bookmark_command(&mut self, command: Command) {
        self.bookmark_commands.insert(command.id.clone(), command);
    }

    /// ブックマークコマンドをクリア
    pub fn clear_bookmarks(&mut self) {
        self.bookmark_commands.clear();
    }

    /// カテゴリでコマンドを取得
    #[allow(dead_code)]
    pub fn get_commands_by_category(&self, category: &str) -> Vec<Command> {
        self.commands
            .values()
            .filter(|cmd| cmd.category == category)
            .cloned()
            .collect()
    }

    /// プロンプトでコマンドを取得
    pub fn get_commands_by_prompt(&self, prompt: &str) -> Vec<Command> {
        self.commands
            .values()
            .filter(|cmd| cmd.prompt.as_ref().map_or(false, |p| p == prompt))
            .cloned()
            .collect()
    }

    /// コマンドを検証
    pub fn validate_command(&self, command: &Command) -> Result<(), String> {
        // 必須フィールドの検証
        if command.name.trim().is_empty() {
            return Err(
                "Command name is required and cannot be empty or contain only whitespace."
                    .to_string(),
            );
        }
        if command.command.trim().is_empty() {
            return Err(
                "Command content is required and cannot be empty or contain only whitespace."
                    .to_string(),
            );
        }
        if command.category.trim().is_empty() {
            return Err(
                "Command category is required and cannot be empty or contain only whitespace."
                    .to_string(),
            );
        }

        // 名前の長さ制限
        if command.name.len() > 100 {
            return Err(format!(
                "Command name is too long ({} characters). Maximum allowed is 100 characters.",
                command.name.len()
            ));
        }

        // カテゴリの長さ制限
        if command.category.len() > 50 {
            return Err(format!(
                "Command category is too long ({} characters). Maximum allowed is 50 characters.",
                command.category.len()
            ));
        }

        // コマンド内容の長さ制限
        if command.command.len() > 1000 {
            return Err(format!(
                "Command content is too long ({} characters). Maximum allowed is 1000 characters.",
                command.command.len()
            ));
        }

        // 説明の長さ制限
        if command.description.len() > 500 {
            return Err(format!("Command description is too long ({} characters). Maximum allowed is 500 characters.", command.description.len()));
        }

        // プロンプトの検証
        if let Some(ref prompt) = command.prompt {
            if prompt.trim().is_empty() {
                return Err("Prompt cannot be empty if specified. Either provide a valid prompt or leave it blank.".to_string());
            }
            if prompt.len() > 10 {
                return Err(format!(
                    "Prompt is too long ({} characters). Maximum allowed is 10 characters.",
                    prompt.len()
                ));
            }
            // プロンプトに空白文字が含まれていないかチェック
            if prompt.contains(' ') || prompt.contains('\t') || prompt.contains('\n') {
                return Err("Prompt cannot contain whitespace characters (spaces, tabs, or newlines). Use a single word without spaces.".to_string());
            }

            // プロンプトの重複チェック
            if self.is_prompt_used(prompt, Some(&command.id)) {
                return Err(format!(
                    "Prompt '{}' is already used by another command.",
                    prompt
                ));
            }

            // 特殊文字のチェック
            if prompt
                .chars()
                .any(|c| !c.is_alphanumeric() && c != '-' && c != '_')
            {
                return Err(
                    "Prompt can only contain letters, numbers, hyphens (-), and underscores (_)."
                        .to_string(),
                );
            }
        }

        // コマンド内容の基本的な妥当性チェック
        if command.command.starts_with("app://") {
            return Err("app:// protocol is deprecated. Please use valid command.".to_string());
        }

        // Scootコマンドの検証（基本的にはユーザーが追加することはないが、バリデーションとしては必要）
        if command.command.starts_with("scoot://") {
            let valid_scoot_commands = [
                "scoot://add-command",
                "scoot://open-commands",
                "scoot://open-config",
                "scoot://open-readme",
                "scoot://open-log",
                "scoot://reload",
                "scoot://kill",
            ];
            if !valid_scoot_commands.contains(&command.command.as_str()) {
                return Err(format!(
                    "Invalid scoot command '{}'. Valid commands are: {}",
                    command.command,
                    valid_scoot_commands.join(", ")
                ));
            }
        }

        Ok(())
    }

    /// Scootコマンドを初期化
    fn initialize_scoot_commands(&mut self) {
        let commands = vec![
            Command {
                id: "scoot-add-command".to_string(),
                name: "Add Command".to_string(),
                category: "scoot".to_string(),
                command: "scoot://add-command".to_string(),
                description: "Add a new command to the launcher".to_string(),
                prompt: None,
                working_dir: None,
                is_editable: false,
            },
            Command {
                id: "scoot-open-commands".to_string(),
                name: "Open Commands.json".to_string(),
                category: "scoot".to_string(),
                command: "scoot://open-commands".to_string(),
                description: "Open commands.json configuration file".to_string(),
                prompt: None,
                working_dir: None,
                is_editable: false,
            },
            Command {
                id: "scoot-open-config".to_string(),
                name: "Open Config.json".to_string(),
                category: "scoot".to_string(),
                command: "scoot://open-config".to_string(),
                description: "Open config.json configuration file".to_string(),
                prompt: None,
                working_dir: None,
                is_editable: false,
            },
            Command {
                id: "scoot-open-readme".to_string(),
                name: "Open README".to_string(),
                category: "scoot".to_string(),
                command: "scoot://open-readme".to_string(),
                description: "Open application README".to_string(),
                prompt: None,
                working_dir: None,
                is_editable: false,
            },
            Command {
                id: "scoot-open-log".to_string(),
                name: "Open Logs".to_string(),
                category: "scoot".to_string(),
                command: "scoot://open-log".to_string(),
                description: "Open application log directory".to_string(),
                prompt: None,
                working_dir: None,
                is_editable: false,
            },
            Command {
                id: "scoot-reload".to_string(),
                name: "Reload Commands".to_string(),
                category: "scoot".to_string(),
                command: "scoot://reload".to_string(),
                description: "Reload commands and configuration".to_string(),
                prompt: None,
                working_dir: None,
                is_editable: false,
            },
            Command {
                id: "scoot-kill".to_string(),
                name: "Kill Scoot".to_string(),
                category: "scoot".to_string(),
                command: "scoot://kill".to_string(),
                description: "Terminate the application".to_string(),
                prompt: None,
                working_dir: None,
                is_editable: false,
            },
        ];

        for command in commands {
            self.scoot_commands.insert(command.id.clone(), command);
        }
    }

    /// プロンプトの重複チェック
    pub fn is_prompt_used(&self, prompt: &str, exclude_id: Option<&str>) -> bool {
        self.commands.values().any(|cmd| {
            if let Some(exclude) = exclude_id {
                if cmd.id == exclude {
                    return false;
                }
            }
            cmd.prompt.as_ref().map_or(false, |p| p == prompt)
        })
    }

    /// カテゴリ一覧を取得
    #[allow(dead_code)]
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .commands
            .values()
            .map(|cmd| cmd.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    /// 指定されたカテゴリのコマンド数を取得
    #[allow(dead_code)]
    pub fn count_by_category(&self, category: &str) -> usize {
        self.commands
            .values()
            .filter(|cmd| cmd.category == category)
            .count()
    }

    /// 全てのユーザー定義コマンドをクリア
    pub fn clear_user_commands(&mut self) {
        self.commands.clear();
    }

    /// アプリケーションコマンドリストを一括更新
    pub fn set_application_commands(&mut self, commands: Vec<Command>) {
        self.application_commands.clear();
        for command in commands {
            self.application_commands
                .insert(command.id.clone(), command);
        }
    }

    /// ブックマークコマンドを一括設定
    pub fn set_bookmark_commands(&mut self, commands: Vec<Command>) {
        self.bookmark_commands.clear();
        for command in commands {
            self.bookmark_commands.insert(command.id.clone(), command);
        }
    }

    /// ユーザーコマンドを一括設定 (検証込み)
    pub fn set_user_commands(&mut self, commands: Vec<Command>) {
        self.commands.clear();
        for command in commands {
            // カテゴリの検証
            if self.validate_command(&command).is_ok() {
                self.add_command(command);
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

    fn create_dummy_command(id: &str, prompt: Option<&str>) -> Command {
        Command {
            id: id.to_string(),
            name: "Test Command".to_string(),
            category: "general".to_string(),
            command: "echo test".to_string(),
            description: "Test".to_string(),
            prompt: prompt.map(|s| s.to_string()),
            working_dir: None,
            is_editable: true,
        }
    }

    #[test]
    fn test_duplicate_prompt_check() {
        let mut manager = CommandManager::new();

        // 1. コマンドAを追加 (prompt: p1)
        let cmd_a = create_dummy_command("cmd-a", Some("p1"));
        assert!(manager.validate_command(&cmd_a).is_ok());
        manager.add_command(cmd_a.clone());

        // 2. コマンドBを追加 (prompt: p1) -> 重複エラー
        let cmd_b_dup = create_dummy_command("cmd-b", Some("p1"));
        let res = manager.validate_command(&cmd_b_dup);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("already used"));

        // 3. コマンドBを追加 (prompt: p2) -> 成功
        let cmd_b = create_dummy_command("cmd-b", Some("p2"));
        assert!(manager.validate_command(&cmd_b).is_ok());
        manager.add_command(cmd_b.clone());

        // 4. コマンドAを更新 (prompt: p2) -> 重複エラー
        let mut cmd_a_update = cmd_a.clone();
        cmd_a_update.prompt = Some("p2".to_string());
        let res = manager.validate_command(&cmd_a_update);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("already used"));

        // 5. コマンドAを更新 (prompt: p1) -> 成功 (自分自身)
        let mut cmd_a_same = cmd_a.clone();
        cmd_a_same.description = "Updated".to_string();
        assert!(manager.validate_command(&cmd_a_same).is_ok());
    }
}
