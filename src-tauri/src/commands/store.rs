use crate::commands::domain::Command;
use crate::json::FileText;

// --- File Storage ---

pub struct CommandStore {
    commands_path: String,
}

impl CommandStore {
    pub(crate) fn new() -> Self {
        Self::in_dir(&crate::os::app_dir())
    }

    pub fn in_dir(dir: &std::path::Path) -> Self {
        Self {
            commands_path: dir.join("commands.json").to_string_lossy().into_owned(),
        }
    }

    /// Load commands. A missing file is created empty; a blank one just means no commands, and
    /// is left for the user to finish writing.
    pub async fn load(&self) -> Result<Vec<Command>, crate::error::AppError> {
        let content = match crate::json::read_text(&self.commands_path).await? {
            FileText::Missing => {
                self.save(&[]).await?;
                return Ok(Vec::new());
            }
            FileText::Blank => return Ok(Vec::new()),
            FileText::Text(content) => content,
        };

        let mut commands = crate::json::parse::<Vec<Command>>(&content).map_err(|e| {
            crate::error::AppError::System(format!(
                "Failed to parse commands.json: {}. The file may be corrupted.",
                e
            ))
        })?;

        let mut needs_save = false;
        for cmd in &mut commands {
            if cmd.id.is_empty() {
                cmd.id = nanoid::nanoid!();
                needs_save = true;
            }
        }
        if needs_save {
            let _ = self.save(&commands).await;
        }

        Ok(commands)
    }

    /// Save commands (strips runtime-only fields like source before writing)
    pub async fn save(&self, commands: &[Command]) -> Result<(), crate::error::AppError> {
        let mut commands_to_save = commands.to_vec();
        for cmd in &mut commands_to_save {
            cmd.source = String::new();
        }

        crate::json::write_pretty(&self.commands_path, &commands_to_save).await
    }

    /// Get commands file path
    pub fn get_path(&self) -> &str {
        &self.commands_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn open_with_is_saved_and_source_is_not() {
        let dir = std::env::temp_dir().join(format!("scoot-store-{}", nanoid::nanoid!()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = CommandStore::in_dir(&dir);
        let mut cmd = Command::new("Private", "url", "user", "https://example.com", "");
        cmd.id = "a".to_string();
        cmd.open_with = Some("chrome.exe".to_string());
        cmd.open_args = Some("--incognito".to_string());
        store.save(&[cmd]).await.unwrap();

        let text = std::fs::read_to_string(store.get_path()).unwrap();
        assert!(!text.contains("source"));
        let loaded = store.load().await.unwrap();
        assert_eq!(loaded[0].open_with.as_deref(), Some("chrome.exe"));
        assert_eq!(loaded[0].open_args.as_deref(), Some("--incognito"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
