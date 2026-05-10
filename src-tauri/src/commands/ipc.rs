use crate::commands::domain::{Command, Commands};
use crate::error::AppError;
use crate::state::AppState;
use tauri::{Manager, State};

/// Get all commands
#[tauri::command]
pub fn get_all_commands(
    state: State<'_, AppState>,
) -> Result<Vec<Command>, crate::error::AppError> {
    let manager = state.commands.lock().unwrap();
    Ok(manager.get_all_commands())
}

/// Add command
#[tauri::command]
pub async fn add_command(
    mut command: Command,
    state: State<'_, AppState>,
) -> Result<String, crate::error::AppError> {
    command.source = crate::commands::domain::SOURCE_USER.to_string();
    let (id, commands) = {
        let mut manager = state
            .commands
            .lock()
            .map_err(|e| AppError::lock(e))?;
        // Validate
        manager.validate_command(&command)?;

        // Add command
        let id = manager.add_user_command(command);
        let commands = manager.get_user_commands();
        (id, commands)
    };

    // Save to config file
    state.command_store.save(&commands).await?;

    Ok(id)
}

/// Update command
#[tauri::command]
pub async fn update_command(
    mut command: Command,
    state: State<'_, AppState>,
) -> Result<(), crate::error::AppError> {
    command.source = crate::commands::domain::SOURCE_USER.to_string();
    let commands = {
        let mut manager = state
            .commands
            .lock()
            .map_err(|e| AppError::lock(e))?;
        // Validate
        manager.validate_command(&command)?;

        // Update command
        manager.update_user_command(command)?;
        manager.get_user_commands()
    };

    // Save to config file
    state.command_store.save(&commands).await?;

    Ok(())
}

/// Delete command
#[tauri::command]
pub async fn delete_command(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), crate::error::AppError> {
    let commands = {
        let mut manager = state
            .commands
            .lock()
            .map_err(|e| AppError::lock(e))?;
        // Delete command
        manager.delete_user_command(&id)?;
        manager.get_user_commands()
    };

    // Save to config file
    state.command_store.save(&commands).await?;

    Ok(())
}

/// Search commands by prompt
#[tauri::command]
pub fn get_commands_by_prompt(
    prompt: String,
    state: State<'_, AppState>,
) -> Result<Vec<Command>, crate::error::AppError> {
    let manager = state.commands.lock().unwrap();
    Ok(manager.get_commands_by_prompt(&prompt))
}

/// Get user commands
#[tauri::command]
pub async fn get_user_commands(state: State<'_, AppState>) -> Result<Commands, crate::error::AppError> {
    state.command_store.load().await
}

/// Save commands
#[tauri::command]
pub async fn save_commands(
    commands: Commands,
    state: State<'_, AppState>,
) -> Result<(), crate::error::AppError> {
    // Save the commands
    state.command_store.save(&commands).await?;
    // Update CommandRegistry
    let mut manager = state
        .commands
        .lock()
        .map_err(|e| AppError::lock(e))?;
    manager.set_user_commands(commands);
    Ok(())
}

/// Get commands.json path
#[tauri::command]
pub fn get_commands_file_path(
    state: State<'_, AppState>,
) -> Result<String, crate::error::AppError> {
    Ok(state.command_store.get_path().to_string())
}

/// Open commands.json
#[tauri::command]
pub async fn open_commands_json(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::error::AppError> {
    if let Some(state) = app_handle.try_state::<crate::state::AppState>() {
        let commands_path = state.command_store.get_path().to_string();
        let _ = state.command_store.load().await; // creates file with defaults if absent
        crate::os::open_path(&app_handle, &commands_path)?;
    }
    Ok(())
}

/// Ignore a command and hide it from future results
#[tauri::command]
pub async fn ignore_command(
    command_path: String,
    app_handle: tauri::AppHandle,
) -> Result<(), crate::error::AppError> {
    log::info!("Ignoring command: {}", command_path);
    if let Some(state) = app_handle.try_state::<crate::state::AppState>() {
        let config_to_save = {
            let mut config = state
                .config
                .lock()
                .map_err(|e| AppError::lock(e))?;
            if !config.ignored.contains(&command_path) {
                config.ignored.push(command_path.clone());
                Some(config.clone())
            } else {
                None
            }
        };

        if let Some(config) = config_to_save {
            // Save config
            state.config_store.save(&config).await?;

            let _ = crate::commands::loader::reload(&state.command_store, &state.commands, &config).await;
        }
    }
    Ok(())
}
