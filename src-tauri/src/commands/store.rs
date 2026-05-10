use crate::commands::domain::Command;

// --- File Storage ---

pub struct CommandStore {
    commands_path: String,
}

impl CommandStore {
    pub fn new() -> Self {
        let target_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        Self {
            commands_path: format!("{}/commands.json", target_dir),
        }
    }

    /// Load commands
    pub async fn load(&self) -> Result<Vec<Command>, crate::error::AppError> {
        // If commands.json does not exist, save the default value and return it
        if !tokio::fs::try_exists(&self.commands_path)
            .await
            .unwrap_or(false)
        {
            return self.save_default_commands().await;
        }

        // Read commands.json
        let content = tokio::fs::read_to_string(&self.commands_path)
            .await
            .map_err(|e| {
                crate::error::AppError::System(format!(
                    "Failed to read commands file '{}': {}",
                    self.commands_path, e
                ))
            })?;

        // If commands.json is empty, save the default value and return it
        if content.trim().is_empty() {
            return self.save_default_commands().await;
        }

        // Validate and parse commands.json
        let commands = tokio::task::spawn_blocking(move || {
            crate::commands::domain::deserialize_json(&content)
        })
        .await
        .map_err(|e| {
            crate::error::AppError::System(format!("Failed to spawn blocking task: {}", e))
        })?
        .unwrap_or_else(|e| {
            log::error!("Failed to parse commands.json: {}", e);
            Vec::new()
        });

        Ok(commands)
    }

    /// Save commands (strips internal fields like id and source before writing)
    pub async fn save(&self, commands: &Vec<Command>) -> Result<(), crate::error::AppError> {
        let mut commands_to_save = commands.clone();
        for cmd in &mut commands_to_save {
            cmd.id = String::new();
            cmd.source = String::new();
        }

        let content = serde_json::to_string_pretty(&commands_to_save).map_err(|e| {
            crate::error::AppError::System(format!("Failed to serialize data: {}", e))
        })?;

        tokio::fs::write(&self.commands_path, content)
            .await
            .map_err(|e| crate::error::AppError::System(format!("Failed to write to file: {}", e)))
    }

    /// Initialize default commands
    async fn save_default_commands(&self) -> Result<Vec<Command>, crate::error::AppError> {
        let default_commands: Vec<Command> = Vec::new();
        self.save(&default_commands).await?;
        Ok(default_commands)
    }

    /// Get commands file path
    pub fn get_path(&self) -> &str {
        &self.commands_path
    }
}
