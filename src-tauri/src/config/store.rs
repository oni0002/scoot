use crate::config::domain::Config;
use crate::error::AppError;
use crate::json::FileText;

// --- Infrastructure / Core Logic ---

pub struct ConfigStore {
    config_path: String, // Config file path
}

/// Config store
/// Load, save, and validate config.json
impl ConfigStore {
    pub(crate) fn new() -> Self {
        Self {
            config_path: crate::os::app_dir()
                .join("config.json")
                .to_string_lossy()
                .into_owned(),
        }
    }

    /// Load the config. A missing file is created with the defaults; a blank one just means the
    /// defaults, and is left for the user to finish writing.
    pub async fn load(&self) -> Result<Config, crate::error::AppError> {
        match crate::json::read_text(&self.config_path).await? {
            FileText::Missing => {
                let default_config = Config::default();
                self.save(&default_config).await?;
                Ok(default_config)
            }
            FileText::Blank => Ok(Config::default()),
            FileText::Text(content) => Config::from_json_with_validation(&content).map_err(|e| {
                crate::error::AppError::System(format!(
                    "App config file '{}' validation failed: {}",
                    self.config_path, e
                ))
            }),
        }
    }

    /// Save the config
    pub async fn save(
        &self,
        config: &crate::config::domain::Config,
    ) -> Result<(), crate::error::AppError> {
        crate::json::write_pretty(&self.config_path, config).await
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
        let mut locked_config = state_config.lock().map_err(AppError::lock)?;
        *locked_config = new_config.clone();
    }

    Ok(new_config)
}
