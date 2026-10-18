use crate::error::AppError;
use crate::settings::domain::Settings;
use crate::state::{CommandsState, SettingsState};
use tauri::{Manager, State};

// --- Tauri Commands ---

/// Get settings
#[tauri::command]
pub async fn get_settings(
    state: State<'_, SettingsState>,
) -> Result<Settings, crate::error::AppError> {
    state
        .settings
        .lock()
        .map(|c| c.clone())
        .map_err(AppError::lock)
}

/// Save settings
#[tauri::command]
pub async fn save_settings(
    settings: Settings,
    state: State<'_, SettingsState>,
) -> Result<(), crate::error::AppError> {
    // Refuse values settings.json could not load back, before they reach the state or the file
    settings.validate()?;

    // state
    {
        let mut locked_settings = state.settings.lock().map_err(AppError::lock)?;
        *locked_settings = settings.clone();
    }

    // settings.json
    state.settings_store.save(&settings).await
}

/// Ignore a command: persist to settings and remove from registry immediately
#[tauri::command]
pub async fn ignore_command(
    command_str: String,
    settings_state: State<'_, SettingsState>,
    commands_state: State<'_, CommandsState>,
    app_handle: tauri::AppHandle,
) -> Result<(), AppError> {
    use tauri::Emitter;

    let settings_snapshot = {
        let mut settings = settings_state.settings.lock().map_err(AppError::lock)?;
        if settings.ignored.contains(&command_str) {
            return Ok(());
        }
        settings.ignored.push(command_str.clone());
        settings.clone()
    };

    settings_state
        .settings_store
        .save(&settings_snapshot)
        .await?;

    {
        let mut registry = commands_state.commands.lock().map_err(AppError::lock)?;
        registry.remove_external_by_command(&command_str);
    }

    // The registry is already up to date, so the webview only has to refetch; no rescan of the
    // sources is needed (the settings file watcher still triggers one for the new ignored list).
    if let Err(e) = app_handle.emit("settings-reloaded", ()) {
        log::error!("Failed to emit settings-reloaded after ignore: {}", e);
    }

    Ok(())
}

/// Undo `ignore_command`: drop it from the ignored list and reload so the command comes back
#[tauri::command]
pub async fn unignore_command(
    command_str: String,
    settings_state: State<'_, SettingsState>,
    app_handle: tauri::AppHandle,
) -> Result<(), AppError> {
    use tauri::Emitter;

    let settings_snapshot = {
        let mut settings = settings_state.settings.lock().map_err(AppError::lock)?;
        if !settings.ignored.contains(&command_str) {
            return Ok(());
        }
        settings.ignored.retain(|c| c != &command_str);
        settings.clone()
    };

    settings_state
        .settings_store
        .save(&settings_snapshot)
        .await?;

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
    settings_state: State<'_, SettingsState>,
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

    let settings_snapshot = {
        let mut settings = settings_state.settings.lock().map_err(AppError::lock)?;
        match &alias {
            Some(a) => settings.aliases.insert(command_str, a.clone()),
            None => settings.aliases.remove(&command_str),
        };
        settings.clone()
    };

    settings_state
        .settings_store
        .save(&settings_snapshot)
        .await?;

    if let Err(e) = app_handle.emit("settings-reloaded", ()) {
        log::error!(
            "Failed to emit settings-reloaded after setting an alias: {}",
            e
        );
    }

    Ok(())
}

/// Open settings.json
#[tauri::command]
pub async fn open_config_json(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    if let Some(state) = app_handle.try_state::<crate::state::SettingsState>() {
        let _ = state.settings_store.load().await; // creates file with defaults if absent
        let settings_path = state.settings_store.file_in_use().to_string();
        crate::os::open_path(&app_handle, &settings_path)?;
    }
    Ok(())
}
