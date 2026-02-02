use crate::domain::command::Command;
use std::collections::HashMap;
use uuid::Uuid;

/// CommandManagerの構造体
pub struct CommandManager {
    // ユーザ定義コマンド
    pub user_commands: HashMap<String, Command>,
    // ブックマークコマンド
    pub bookmark_commands: HashMap<String, Command>,
    // Scootコマンド
    pub scoot_commands: HashMap<String, Command>,
    // アプリケーションコマンド
    pub application_commands: HashMap<String, Command>,
}

/// commandを管理するためのクラス
impl CommandManager {
    pub fn new() -> Self {
        Self {
            user_commands: HashMap::new(),
            bookmark_commands: HashMap::new(),
            scoot_commands: HashMap::new(),
            application_commands: HashMap::new(),
        }
    }

    /// Scootコマンドを一括設定
    pub fn set_scoot_commands(&mut self, commands: Vec<Command>) {
        self.scoot_commands.clear();
        for command in commands {
            self.scoot_commands.insert(command.id.clone(), command);
        }
    }

    /// ユーザコマンドを追加
    pub fn add_user_command(&mut self, mut command: Command) -> String {
        // IDが空の場合は新しいUUIDを生成
        if command.id.is_empty() {
            command.id = Uuid::new_v4().to_string();
        }
        // ID重複チェック
        let mut final_id = command.id.clone();
        let mut counter = 1;
        while self.user_commands.contains_key(&final_id) {
            final_id = format!("{}-{}", command.id, counter);
            counter += 1;
        }
        command.id = final_id.clone();

        // コマンド一覧に追加
        self.user_commands.insert(final_id.clone(), command);
        final_id
    }

    /// ユーザコマンドを更新
    pub fn update_user_command(&mut self, command: Command) -> Result<(), String> {
        if !self.user_commands.contains_key(&command.id) {
            return Err("Command not found".to_string());
        }

        self.user_commands.insert(command.id.clone(), command);
        Ok(())
    }

    /// ユーザコマンドを削除
    pub fn delete_user_command(&mut self, id: &str) -> Result<(), String> {
        if self.user_commands.remove(id).is_none() {
            return Err("Command not found".to_string());
        }
        Ok(())
    }

    /// IDでコマンドを取得
    #[allow(dead_code)]
    pub fn get_command(&self, id: &str) -> Option<&Command> {
        self.user_commands
            .get(id)
            .or_else(|| self.bookmark_commands.get(id))
            .or_else(|| self.scoot_commands.get(id))
            .or_else(|| self.application_commands.get(id))
    }

    /// 全てのコマンドを取得
    pub fn get_all_commands(&self) -> Vec<Command> {
        let mut all_commands = Vec::new();
        all_commands.extend(self.user_commands.values().cloned());
        all_commands.extend(self.bookmark_commands.values().cloned());
        all_commands.extend(self.scoot_commands.values().cloned());
        all_commands.extend(self.application_commands.values().cloned());
        all_commands
    }

    /// ユーザー定義コマンドのみ取得 (commands.json用)
    pub fn get_user_commands(&self) -> Vec<Command> {
        self.user_commands.values().cloned().collect()
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
        self.user_commands
            .values()
            .filter(|cmd| cmd.category == category)
            .cloned()
            .collect()
    }

    /// プロンプトでコマンドを取得
    pub fn get_commands_by_prompt(&self, prompt: &str) -> Vec<Command> {
        self.user_commands
            .values()
            .filter(|cmd| cmd.prompt.as_ref().map_or(false, |p| p == prompt))
            .cloned()
            .collect()
    }

    /// コマンドを検証
    pub fn validate_command(&self, command: &Command) -> Result<(), String> {
        // ドメインレベルの検証 (フォーマット等)
        command.validate()?;

        // ストアレベルの検証 (プロンプトのユニーク性)
        if let Some(ref prompt) = command.prompt {
            if self.is_prompt_used(prompt, Some(&command.id)) {
                return Err(format!(
                    "Prompt '{}' is already used by another command.",
                    prompt
                ));
            }
        }

        Ok(())
    }

    /// プロンプトの重複チェック
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

    /// カテゴリ一覧を取得
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

    /// 指定されたカテゴリのコマンド数を取得
    #[allow(dead_code)]
    pub fn count_by_category(&self, category: &str) -> usize {
        self.user_commands
            .values()
            .filter(|cmd| cmd.category == category)
            .count()
    }

    /// 全てのユーザー定義コマンドをクリア
    pub fn clear_user_commands(&mut self) {
        self.user_commands.clear();
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
        self.user_commands.clear();
        for command in commands {
            // カテゴリの検証
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

    fn create_dummy_command(id: &str, prompt: Option<&str>) -> Command {
        Command {
            id: id.to_string(),
            name: "Test Command".to_string(),
            category: "general".to_string(),
            command: "echo test".to_string(),
            description: "Test".to_string(),
            prompt: prompt.map(|s| s.to_string()),
            working_dir: None,
            show_window: None,
            is_editable: true,
        }
    }

    #[test]
    fn test_duplicate_prompt_check() {
        let mut manager = CommandManager::new();

        // 1. コマンドAを追加 (prompt: p1)
        let cmd_a = create_dummy_command("cmd-a", Some("p1"));
        assert!(manager.validate_command(&cmd_a).is_ok());
        manager.add_user_command(cmd_a.clone());

        // 2. コマンドBを追加 (prompt: p1) -> 重複エラー
        let cmd_b_dup = create_dummy_command("cmd-b", Some("p1"));
        let res = manager.validate_command(&cmd_b_dup);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("already used"));

        // 3. コマンドBを追加 (prompt: p2) -> 成功
        let cmd_b = create_dummy_command("cmd-b", Some("p2"));
        assert!(manager.validate_command(&cmd_b).is_ok());
        manager.add_user_command(cmd_b.clone());

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

    #[test]
    fn test_set_scoot_commands() {
        let mut manager = CommandManager::new();
        let scoot_cmd = create_dummy_command("scoot-1", None);

        manager.set_scoot_commands(vec![scoot_cmd.clone()]);

        let all = manager.get_all_commands();
        assert!(all.iter().any(|c| c.id == "scoot-1"));
    }

    #[test]
    fn test_add_user_command_id_generation() {
        let mut manager = CommandManager::new();
        let cmd = create_dummy_command("", None); // Empty ID

        let id = manager.add_user_command(cmd);
        assert!(!id.is_empty());
        assert!(manager.get_user_commands().iter().any(|c| c.id == id));
    }
}
