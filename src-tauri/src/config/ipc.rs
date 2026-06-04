use crate::config::domain::Config;
use crate::error::AppError;
use crate::state::{CommandsState, ConfigState};
use tauri::{Manager, State};

// --- Tauri Commands ---

/// Get config
#[tauri::command]
pub async fn get_config(state: State<'_, ConfigState>) -> Result<Config, crate::error::AppError> {
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
    state: State<'_, ConfigState>,
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

/// Ignore a command: persist to config and remove from registry immediately
#[tauri::command]
pub async fn ignore_command(
    command_str: String,
    config_state: State<'_, ConfigState>,
    commands_state: State<'_, CommandsState>,
    app_handle: tauri::AppHandle,
) -> Result<(), AppError> {
    use tauri::Emitter;

    let config_snapshot = {
        let mut config = config_state.config.lock().map_err(|e| AppError::lock(e))?;
        if config.ignored.contains(&command_str) {
            return Ok(());
        }
        config.ignored.push(command_str.clone());
        config.clone()
    };

    config_state.config_store.save(&config_snapshot).await?;

    {
        let mut registry = commands_state.commands.lock().map_err(|e| AppError::lock(e))?;
        registry.external_commands.retain(|c| c.command != command_str);
    }

    if let Err(e) = app_handle.emit("request-reload", ()) {
        log::error!("Failed to emit request-reload after ignore: {}", e);
    }

    Ok(())
}

/// Open config.json
#[tauri::command]
pub async fn open_config_json(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    if let Some(state) = app_handle.try_state::<crate::state::ConfigState>() {
        let config_path = state.config_store.get_config_path().to_string();
        let _ = state.config_store.load().await; // creates file with defaults if absent
        crate::os::open_path(&app_handle, &config_path)?;
    }
    Ok(())
}
