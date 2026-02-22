use crate::domain::config::Config;
use crate::infra::config::ConfigManager;

/// 設定を取得
pub fn get(config: &std::sync::Mutex<Config>) -> Result<Config, crate::domain::error::AppError> {
    config
        .lock()
        .map(|c| c.clone())
        .map_err(|e| crate::domain::error::AppError::System(e.to_string()))
}

/// 設定を保存
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

/// 設定ファイルのパスを取得
pub fn get_file_path(config_manager: &ConfigManager) -> String {
    config_manager.get_config_path().to_string()
}

/// 設定を読み込み、メモリ上の設定を更新
pub async fn reload(
    config_manager: &ConfigManager,
    state_config: &std::sync::Mutex<Config>,
) -> Result<Config, crate::domain::error::AppError> {
    // 設定の読み込み
    log::info!("Loading configuration.");
    let new_config = config_manager
        .load()
        .await
        .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;

    // メモリ上のConfigを更新
    {
        let mut locked_config = state_config
            .lock()
            .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
        *locked_config = new_config.clone();
    }

    Ok(new_config)
}
