use crate::config::domain::Config;
use crate::error::AppError;
use crate::state::AppState;
use tauri::{Manager, State};

// --- Tauri Commands ---

/// Get config
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Config, crate::error::AppError> {
    state
        .config
        .lock()
        .map(|c| c.clone())
        .map_err(|e| AppError::lock(e))
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
            .map_err(|e| AppError::lock(e))?;
        *locked_config = config.clone();
    }

    // config.json
    state.config_store.save(&config).await
}

/// Get config.json path
#[tauri::command]
pub async fn get_config_file_path(
    state: State<'_, AppState>,
) -> Result<String, crate::error::AppError> {
    Ok(state.config_store.get_config_path().to_string())
}

/// Open config.json
#[tauri::command]
pub async fn open_config_json(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    if let Some(state) = app_handle.try_state::<crate::state::AppState>() {
        let config_path = state.config_store.get_config_path().to_string();
        let _ = state.config_store.load().await; // creates file with defaults if absent
        crate::system::open_path(&app_handle, &config_path)?;
    }
    Ok(())
}
