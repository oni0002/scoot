use crate::config::domain::Config;
use crate::state::AppState;
use serde_json;
use tauri::State;

// --- Tauri Commands ---

/// Get config
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Config, crate::error::AppError> {
    state
        .config
        .lock()
        .map(|c| c.clone())
        .map_err(|e| crate::error::AppError::System(e.to_string()))
}

/// Save config
#[tauri::command]
pub async fn save_config(
    config: Config,
    state: State<'_, AppState>,
) -> Result<(), crate::error::AppError> {
    // state
    {
        let mut locked_config = state
            .config
            .lock()
            .map_err(|e| crate::error::AppError::System(e.to_string()))?;
        *locked_config = config.clone();
    }

    // config.json
    state.config_manager.save(&config).await
}

/// Get config.json path
#[tauri::command]
pub async fn get_config_file_path(
    state: State<'_, AppState>,
) -> Result<String, crate::error::AppError> {
    Ok(state.config_manager.get_config_path().to_string())
}

/// Get config.json schema
#[tauri::command]
pub async fn get_config_schema() -> Result<serde_json::Value, crate::error::AppError> {
    Ok(Config::generate_schema())
}

/// Validate config.json
#[tauri::command]
pub async fn validate_config(
    config: serde_json::Value,
) -> Result<serde_json::Value, crate::error::AppError> {
    match Config::from_json_with_validation(&config.to_string()) {
        Ok(_) => Ok(serde_json::json!({ "valid": true, "errors": [] })),
        Err(error) => Ok(serde_json::json!({ "valid": false, "errors": [error] })),
    }
}

/// Open config.json
#[tauri::command]
pub async fn open_config_json(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    crate::system::open_config_json(&app_handle)
}
