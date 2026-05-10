use crate::config::domain::Config;
use crate::error::AppError;

// --- Infrastructure / Core Logic ---

pub struct ConfigStore {
    config_path: String, // Config file path
}

/// Config store
/// Load, save, and validate config.json
impl ConfigStore {
    pub fn new() -> Self {
        // Get the target directory
        let target_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        log::debug!("App path: {}", target_dir);

        Self {
            config_path: format!("{}/config.json", target_dir),
        }
    }

    /// Load the config
    pub async fn load(&self) -> Result<Config, crate::error::AppError> {
        // If config.json does not exist, save the default value and return it
        if !tokio::fs::try_exists(&self.config_path)
            .await
            .unwrap_or(false)
        {
            return self.save_default_config().await;
        }

        // Read config.json
        let content = tokio::fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| {
                crate::error::AppError::System(format!(
                    "Failed to read app config file '{}': {}",
                    self.config_path, e
                ))
            })?;
        // If config.json is empty, save the default value and return it
        if content.trim().is_empty() {
            return self.save_default_config().await;
        }

        // Parse config.json
        let config: Config = Config::from_json_with_validation(&content).map_err(|e| {
            crate::error::AppError::System(format!(
                "App config file '{}' validation failed: {}",
                self.config_path, e
            ))
        })?;

        // Auto-upgrade format to camelCase by saving the loaded config back to the file
        // Ignore save errors during auto-upgrade
        let new_content = serde_json::to_string_pretty(&config).unwrap_or_default();
        if content != new_content {
            let _ = self.save(&config).await;
        }

        Ok(config)
    }

    /// Initialize default config
    async fn save_default_config(&self) -> Result<Config, crate::error::AppError> {
        let default_config = Config::default();
        self.save(&default_config).await?;
        Ok(default_config)
    }

    /// Save the config
    pub async fn save(
        &self,
        config: &crate::config::domain::Config,
    ) -> Result<(), crate::error::AppError> {
        let content = serde_json::to_string_pretty(config).map_err(|e| {
            crate::error::AppError::System(format!("Failed to serialize config: {}", e))
        })?;

        tokio::fs::write(&self.config_path, content)
            .await
            .map_err(|e| crate::error::AppError::System(format!("Failed to write to file: {}", e)))
    }

    /// Get config file path
    pub fn get_config_path(&self) -> &str {
        &self.config_path
    }
}

/// Reload config
pub async fn reload(
    config_store: &ConfigStore,
    state_config: &std::sync::Mutex<Config>,
) -> Result<Config, crate::error::AppError> {
    // Load config
    log::debug!("Loading configuration.");
    let new_config = config_store.load().await?;

    // Update state config
    {
        let mut locked_config = state_config
            .lock()
            .map_err(|e| AppError::lock(e))?;
        *locked_config = new_config.clone();
    }

    Ok(new_config)
}
