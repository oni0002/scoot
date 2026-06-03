use crate::commands::domain::Command;
use crate::error::AppError;
use crate::state::CommandsState;
use tauri::{Manager, State};

/// Get all commands
#[tauri::command]
pub fn get_all_commands(
    state: State<'_, CommandsState>,
) -> Result<Vec<Command>, crate::error::AppError> {
    let manager = state.commands.lock().map_err(|e| AppError::lock(e))?;
    Ok(manager.get_all_commands())
}

/// Add command
#[tauri::command]
pub async fn add_command(
    mut command: Command,
    state: State<'_, CommandsState>,
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
    state: State<'_, CommandsState>,
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
    state: State<'_, CommandsState>,
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

/// Search commands by alias
#[tauri::command]
pub fn get_commands_by_alias(
    alias: String,
    state: State<'_, CommandsState>,
) -> Result<Vec<Command>, crate::error::AppError> {
    let manager = state.commands.lock().map_err(|e| AppError::lock(e))?;
    Ok(manager.get_commands_by_alias(&alias))
}

/// Open commands.json
#[tauri::command]
pub async fn open_commands_json(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::error::AppError> {
    if let Some(state) = app_handle.try_state::<crate::state::CommandsState>() {
        let commands_path = state.command_store.get_path().to_string();
        let _ = state.command_store.load().await; // creates file with defaults if absent
        crate::os::open_path(&app_handle, &commands_path)?;
    }
    Ok(())
}

