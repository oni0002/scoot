use crate::commands::domain::{Command, Commands};
use crate::commands::store::CommandRegistry;
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
            .map_err(|e| crate::error::AppError::System(e.to_string()))?;
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
            .map_err(|e| crate::error::AppError::System(e.to_string()))?;
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
            .map_err(|e| crate::error::AppError::System(e.to_string()))?;
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

/// Get commands
#[tauri::command]
pub async fn get_commands(state: State<'_, AppState>) -> Result<Commands, crate::error::AppError> {
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
        .map_err(|e| crate::error::AppError::System(e.to_string()))?;
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

/// Get commands.json schema
#[tauri::command]
pub fn get_commands_schema() -> Result<serde_json::Value, crate::error::AppError> {
    Ok(crate::commands::domain::generate_commands_schema())
}

/// Validate commands.json
#[tauri::command]
pub fn validate_commands(
    config: serde_json::Value,
) -> Result<serde_json::Value, crate::error::AppError> {
    match crate::commands::domain::deserialize_json(&config.to_string()) {
        Ok(_) => Ok(serde_json::json!({ "valid": true, "errors": [] })),
        Err(error) => Ok(serde_json::json!({ "valid": false, "errors": [error] })),
    }
}

/// Open commands.json
#[tauri::command]
pub async fn open_commands_json(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::error::AppError> {
    if let Some(state) = app_handle.try_state::<crate::state::AppState>() {
        let commands_path = state.command_store.get_path();

        let path = std::path::Path::new(&commands_path);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| crate::error::AppError::System(e.to_string()))?;
                }
            }
            let default_commands = crate::commands::domain::Commands::default();
            let _ = state.command_store.save(&default_commands).await;
        }

        crate::system::open_path(&app_handle, &commands_path)?;
    }
    Ok(())
}

/// Reload commands, bookmarks, and apps
pub async fn reload(
    command_store: &crate::commands::store::CommandStore,
    command_registry: &std::sync::Mutex<CommandRegistry>,
    config: &crate::config::domain::Config,
) -> Result<(), crate::error::AppError> {
    // Load commands
    log::debug!("Loading commands");
    let mut commands = match command_store.load().await {
        Ok(cmds) => cmds,
        Err(e) => {
            log::error!(
                "Failed to load commands.json: {}. Proceeding with empty commands.",
                e
            );
            crate::commands::domain::Commands::new()
        }
    };
    // Assign source to user commands
    for cmd in &mut commands {
        cmd.source = crate::commands::domain::SOURCE_USER.to_string();
    }

    // Load bookmarks
    log::debug!("Loading bookmarks");
    let bookmarks = if config.bookmarks.enabled {
        crate::commands::bookmark::load(&config.bookmarks)
            .await
            .unwrap_or_else(|e| {
                log::warn!("Failed to load bookmarks: {}", e);
                Vec::new()
            })
    } else {
        Vec::new()
    };

    // Scan applications
    log::debug!("Loading applications");
    let app_commands = if config.applications.enabled {
        crate::commands::application::load(
            &config.applications.directories,
            &config.applications.extensions,
        )
        .await
        .unwrap_or_default()
    } else {
        Vec::new()
    };

    // Filter ignored commands
    let bookmarks: Vec<_> = bookmarks
        .into_iter()
        .filter(|c| !config.ignored.contains(&c.command))
        .collect();
    
    let app_commands: Vec<_> = app_commands
        .into_iter()
        .filter(|c| !config.ignored.contains(&c.command))
        .collect();

    // Load markdown links
    log::debug!("Loading markdown links");
    let markdown_commands = if config.markdown.enabled {
        crate::commands::markdown::load(&config.markdown)
            .await
            .unwrap_or_else(|e| {
                log::warn!("Failed to load markdown links: {}", e);
                Vec::new()
            })
    } else {
        Vec::new()
    };

    let markdown_commands: Vec<_> = markdown_commands
        .into_iter()
        .filter(|c| !config.ignored.contains(&c.command))
        .collect();

    // Filter ignored scoot commands
    let scoot_commands: Vec<_> = crate::commands::domain::get_scoot_commands()
        .into_iter()
        .filter(|c| !config.ignored.contains(&c.command))
        .collect();

    // Reflect in CommandRegistry
    let mut manager = command_registry
        .lock()
        .map_err(|e| crate::error::AppError::System(e.to_string()))?;
    manager.set_user_commands(commands);
    manager.set_bookmark_commands(bookmarks);
    manager.set_application_commands(app_commands);
    manager.set_scoot_commands(scoot_commands);
    manager.set_markdown_commands(markdown_commands);

    Ok(())
}

/// Reload bookmarks only
pub async fn reload_bookmarks(
    command_registry: &std::sync::Mutex<CommandRegistry>,
    config: &crate::config::domain::Config,
) -> Result<(), crate::error::AppError> {
    // Load bookmarks
    log::debug!("Loading bookmarks.");
    let bookmarks = if config.bookmarks.enabled {
        crate::commands::bookmark::load(&config.bookmarks)
            .await
            .unwrap_or_else(|e| {
                log::warn!("Failed to load bookmarks: {}", e);
                Vec::new()
            })
    } else {
        Vec::new()
    };

    // Filter ignored commands
    let bookmarks: Vec<_> = bookmarks
        .into_iter()
        .filter(|c| !config.ignored.contains(&c.command))
        .collect();

    // Reflect in CommandRegistry
    let mut manager = command_registry
        .lock()
        .map_err(|e| crate::error::AppError::System(e.to_string()))?;
    manager.set_bookmark_commands(bookmarks);

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
            let mut config = state.config.lock().map_err(|e| crate::error::AppError::System(e.to_string()))?;
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
            
            // Reload registries specifically by calling reload
            let _ = reload(&state.command_store, &state.commands, &config).await;
        }
    }
    Ok(())
}


