use crate::domain::command::Commands;
use crate::domain::config::Config;
use serde_json;

pub struct ConfigManager {
    config_path: String,   // Config file path
    commands_path: String, // Commands file path
}

/// Config manager class
/// Validates, serializes/deserializes, and generates default values for config.json and commands.json
impl ConfigManager {
    pub fn new() -> Self {
        // Get the target directory
        let target_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        log::debug!("Using config: {}", target_dir);

        let config_path = format!("{}/config.json", target_dir);
        let commands_path = format!("{}/commands.json", target_dir);

        Self {
            config_path,
            commands_path,
        }
    }

    /// Load the config
    pub async fn load(&self) -> Result<Config, crate::domain::error::AppError> {
        // If config.json does not exist, save the default value and return it
        if !tokio::fs::try_exists(&self.config_path)
            .await
            .unwrap_or(false)
        {
            let default_config = Config::default();
            self.save(&default_config).await?;
            return Ok(default_config);
        }

        // Read config.json
        let content = tokio::fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| {
                crate::domain::error::AppError::System(format!(
                    "Failed to read app config file '{}': {}",
                    self.config_path, e
                ))
            })?;

        // If config.json is empty, save the default value and return it
        if content.trim().is_empty() {
            let default_config = Config::default();
            self.save(&default_config).await?;
            return Ok(default_config);
        }

        // Parse config.json
        let mut config: Config = Config::from_json_with_validation(&content).map_err(|e| {
            crate::domain::error::AppError::System(format!(
                "App config file '{}' validation failed: {}",
                self.config_path, e
            ))
        })?;

        // Validate and fix fuzzy_threshold
        config.validate_and_fix()?;

        // Auto-upgrade format to camelCase by saving the loaded config back to the file
        // Ignore save errors during auto-upgrade
        let _ = self.save(&config).await;

        Ok(config)
    }

    /// Load commands
    pub async fn load_commands(&self) -> Result<Commands, crate::domain::error::AppError> {
        // If commands.json does not exist, save the default value and return it
        if !tokio::fs::try_exists(&self.commands_path)
            .await
            .unwrap_or(false)
        {
            let default_commands: Commands = Vec::new();
            self.save_commands(&default_commands).await?;
            return Ok(default_commands);
        }

        // Read commands.json
        let content = tokio::fs::read_to_string(&self.commands_path)
            .await
            .map_err(|e| {
                crate::domain::error::AppError::System(format!(
                    "Failed to read commands file '{}': {}",
                    self.commands_path, e
                ))
            })?;

        // If commands.json is empty, save the default value and return it
        if content.trim().is_empty() {
            let default_commands: Commands = Vec::new();
            self.save_commands(&default_commands).await?;
            return Ok(default_commands);
        }

        // Validate and parse commands.json
        // JSON parsing may be a heavy process, so use spawn_blocking
        let commands = tokio::task::spawn_blocking(move || {
            crate::domain::config::commands_from_json_with_validation(&content)
        })
        .await
        .map_err(|e| {
            crate::domain::error::AppError::System(format!("Failed to spawn blocking task: {}", e))
        })?
        .unwrap_or_else(|e| {
            log::error!("Failed to parse commands.json: {}", e);
            Vec::new()
        });

        // Auto-upgrade format to camelCase by saving the loaded commands back to the file
        let _ = self.save_commands(&commands).await;

        Ok(commands)
    }

    /// Save the config
    pub async fn save(
        &self,
        config: &crate::domain::config::Config,
    ) -> Result<(), crate::domain::error::AppError> {
        self.save_to_json(&self.config_path, config).await
    }

    /// Save commands
    pub async fn save_commands(
        &self,
        commands: &crate::domain::command::Commands,
    ) -> Result<(), crate::domain::error::AppError> {
        self.save_to_json(&self.commands_path, commands).await
    }

    /// Save to JSON
    async fn save_to_json<T: serde::Serialize>(
        &self,
        path: &str,
        data: &T,
    ) -> Result<(), crate::domain::error::AppError> {
        let content = serde_json::to_string_pretty(data).map_err(|e| {
            crate::domain::error::AppError::System(format!("Failed to serialize data: {}", e))
        })?;

        tokio::fs::write(path, content).await.map_err(|e| {
            crate::domain::error::AppError::System(format!("Failed to write to file: {}", e))
        })
    }

    /// Get config file path
    pub fn get_config_path(&self) -> &str {
        &self.config_path
    }

    /// Get commands file path
    pub fn get_commands_path(&self) -> &str {
        &self.commands_path
    }
}
