use crate::domain::command::{Command, Commands}; // models::Command -> command::Command (Already likely updated but ensuring import is correct or removing duplication if any)
use crate::infra::config::ConfigManager;
use crate::store::commands::CommandManager;

/// Get the commands
pub async fn get_commands(
    config_manager: &ConfigManager,
) -> Result<Commands, crate::domain::error::AppError> {
    config_manager.load_commands().await
}

/// Save the commands
pub async fn save_commands(
    config_manager: &ConfigManager,
    command_manager: &std::sync::Mutex<CommandManager>,
    commands: &Commands,
) -> Result<(), crate::domain::error::AppError> {
    // Save the commands
    config_manager.save_commands(commands).await?;
    // Update CommandManager
    let mut manager = command_manager
        .lock()
        .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
    manager.set_user_commands(commands.clone());
    Ok(())
}

/// Get the path to the commands file
pub fn get_file_path(config_manager: &ConfigManager) -> String {
    config_manager.get_commands_path().to_string()
}

/// Reload commands, bookmarks, and apps
pub async fn reload(
    config_manager: &ConfigManager,
    command_manager: &std::sync::Mutex<CommandManager>,
    config: &crate::domain::config::Config,
) -> Result<(), crate::domain::error::AppError> {
    // Load commands
    log::debug!("Loading commands.");
    let commands = match config_manager.load_commands().await {
        Ok(cmds) => cmds,
        Err(e) => {
            log::error!(
                "Failed to load commands.json: {}. Proceeding with empty commands.",
                e
            );
            crate::domain::command::Commands::new()
        }
    };

    // Load bookmarks
    let bookmarks = load_internal_bookmarks(&config.bookmarks).await;

    // Scan applications
    log::debug!("Loading applications.");
    let app_commands = if config.applications.enabled {
        crate::infra::application::scan(
            &config.applications.directories,
            &config.applications.extensions,
        )
        .await
        .unwrap_or_default()
    } else {
        Vec::new()
    };

    // Reflect in CommandManager
    let mut manager = command_manager
        .lock()
        .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
    manager.set_user_commands(commands);
    manager.set_bookmark_commands(bookmarks);
    manager.set_application_commands(app_commands);

    Ok(())
}

/// Reload bookmarks only
pub async fn reload_bookmarks(
    command_manager: &std::sync::Mutex<CommandManager>,
    config: &crate::domain::config::Config,
) -> Result<(), crate::domain::error::AppError> {
    // Load bookmarks
    let bookmarks = load_internal_bookmarks(&config.bookmarks).await;

    // Reflect in CommandManager
    let mut manager = command_manager
        .lock()
        .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
    manager.set_bookmark_commands(bookmarks);

    Ok(())
}

/// Helper method to load bookmarks consistently
async fn load_internal_bookmarks(
    bookmark_config: &crate::domain::config::BookmarkConfig,
) -> Vec<Command> {
    log::debug!("Loading bookmarks.");
    if bookmark_config.enabled {
        match crate::infra::bookmark::load(bookmark_config).await {
            Ok(bm_commands) => bm_commands,
            Err(e) => {
                log::warn!("Failed to load bookmarks: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    }
}

/// Get all commands
pub fn get_all(command_manager: &std::sync::Mutex<CommandManager>) -> Vec<Command> {
    let manager = command_manager.lock().unwrap();
    manager.get_all_commands()
}

/// Search commands by prompt
pub fn get_by_prompt(
    command_manager: &std::sync::Mutex<CommandManager>,
    prompt: &str,
) -> Vec<Command> {
    let manager = command_manager.lock().unwrap();
    manager.get_commands_by_prompt(prompt)
}

/// Add a command
pub async fn add(
    config_manager: &ConfigManager,
    command_manager: &std::sync::Mutex<CommandManager>,
    command: Command,
) -> Result<String, crate::domain::error::AppError> {
    let (id, commands) = {
        let mut manager = command_manager
            .lock()
            .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
        // Validate
        manager.validate_command(&command)?;

        // Add command
        let id = manager.add_user_command(command);
        let commands = manager.get_user_commands();
        (id, commands)
    };

    // Save to config file
    config_manager.save_commands(&commands).await?;

    Ok(id)
}

/// Update a command
pub async fn update(
    config_manager: &ConfigManager,
    command_manager: &std::sync::Mutex<CommandManager>,
    command: Command,
) -> Result<(), crate::domain::error::AppError> {
    let commands = {
        let mut manager = command_manager
            .lock()
            .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
        // Validate
        manager.validate_command(&command)?;

        // Update command
        manager.update_user_command(command)?;
        manager.get_user_commands()
    };

    // Save to config file
    config_manager.save_commands(&commands).await?;

    Ok(())
}

/// Delete a command
pub async fn delete(
    config_manager: &ConfigManager,
    command_manager: &std::sync::Mutex<CommandManager>,
    id: &str,
) -> Result<(), crate::domain::error::AppError> {
    let commands = {
        let mut manager = command_manager
            .lock()
            .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
        // Delete command
        manager.delete_user_command(id)?;
        manager.get_user_commands()
    };

    // Save to config file
    config_manager.save_commands(&commands).await?;

    Ok(())
}
