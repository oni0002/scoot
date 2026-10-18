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
        .map_err(AppError::lock)
}

/// Save config
#[tauri::command]
pub async fn save_config(
    config: Config,
    state: State<'_, ConfigState>,
) -> Result<(), crate::error::AppError> {
    // Refuse values settings.json could not load back, before they reach the state or the file
    config.validate()?;

    // state
    {
        let mut locked_config = state.config.lock().map_err(AppError::lock)?;
        *locked_config = config.clone();
    }

    // settings.json
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
        let mut config = config_state.config.lock().map_err(AppError::lock)?;
        if config.ignored.contains(&command_str) {
            return Ok(());
        }
        config.ignored.push(command_str.clone());
        config.clone()
    };

    config_state.config_store.save(&config_snapshot).await?;

    {
        let mut registry = commands_state.commands.lock().map_err(AppError::lock)?;
        registry.remove_external_by_command(&command_str);
    }

    // The registry is already up to date, so the webview only has to refetch; no rescan of the
    // sources is needed (the config file watcher still triggers one for the new ignored list).
    if let Err(e) = app_handle.emit("config-reloaded", ()) {
        log::error!("Failed to emit config-reloaded after ignore: {}", e);
    }

    Ok(())
}

/// Undo `ignore_command`: drop it from the ignored list and reload so the command comes back
#[tauri::command]
pub async fn unignore_command(
    command_str: String,
    config_state: State<'_, ConfigState>,
    app_handle: tauri::AppHandle,
) -> Result<(), AppError> {
    use tauri::Emitter;

    let config_snapshot = {
        let mut config = config_state.config.lock().map_err(AppError::lock)?;
        if !config.ignored.contains(&command_str) {
            return Ok(());
        }
        config.ignored.retain(|c| c != &command_str);
        config.clone()
    };

    config_state.config_store.save(&config_snapshot).await?;

    if let Err(e) = app_handle.emit("request-reload", ()) {
        log::error!("Failed to emit request-reload after unignore: {}", e);
    }

    Ok(())
}

/// Set (or, with `None`, clear) the alias of an external command. Like `ignore_command` it is keyed
/// by the `command` string and written to settings.json at once; the registry is checked first so a
/// taken or malformed alias never reaches the file.
#[tauri::command]
pub async fn set_external_alias(
    command_str: String,
    alias: Option<String>,
    config_state: State<'_, ConfigState>,
    commands_state: State<'_, CommandsState>,
    app_handle: tauri::AppHandle,
) -> Result<(), AppError> {
    use tauri::Emitter;

    let alias = alias
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty());

    {
        let mut registry = commands_state.commands.lock().map_err(AppError::lock)?;
        registry.set_external_alias(&command_str, alias.as_deref())?;
    }

    let config_snapshot = {
        let mut config = config_state.config.lock().map_err(AppError::lock)?;
        match &alias {
            Some(a) => config.aliases.insert(command_str, a.clone()),
            None => config.aliases.remove(&command_str),
        };
        config.clone()
    };

    config_state.config_store.save(&config_snapshot).await?;

    if let Err(e) = app_handle.emit("config-reloaded", ()) {
        log::error!(
            "Failed to emit config-reloaded after setting an alias: {}",
            e
        );
    }

    Ok(())
}

/// Open settings.json
#[tauri::command]
pub async fn open_config_json(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    if let Some(state) = app_handle.try_state::<crate::state::ConfigState>() {
        let _ = state.config_store.load().await; // creates file with defaults if absent
        let config_path = state.config_store.file_in_use().to_string();
        crate::os::open_path(&app_handle, &config_path)?;
    }
    Ok(())
}
