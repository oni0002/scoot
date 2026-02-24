use crate::domain::config::Config;
use crate::infra::config::ConfigManager;

/// Get config
pub fn get(config: &std::sync::Mutex<Config>) -> Result<Config, crate::domain::error::AppError> {
    config
        .lock()
        .map(|c| c.clone())
        .map_err(|e| crate::domain::error::AppError::System(e.to_string()))
}

/// Save config
pub async fn save(
    config_manager: &ConfigManager,
    state_config: &std::sync::Mutex<Config>,
    config: &Config,
) -> Result<(), crate::domain::error::AppError> {
    // stateを更新
    {
        let mut locked_config = state_config
            .lock()
            .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
        *locked_config = config.clone();
    }

    // config.jsonに保存
    config_manager.save(config).await
}

/// Get config file path
pub fn get_file_path(config_manager: &ConfigManager) -> String {
    config_manager.get_config_path().to_string()
}

/// Reload config
pub async fn reload(
    config_manager: &ConfigManager,
    state_config: &std::sync::Mutex<Config>,
) -> Result<Config, crate::domain::error::AppError> {
    // Load config
    log::debug!("Loading configuration.");
    let new_config = config_manager
        .load()
        .await
        .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;

    // Update state config
    {
        let mut locked_config = state_config
            .lock()
            .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
        *locked_config = new_config.clone();
    }

    Ok(new_config)
}
