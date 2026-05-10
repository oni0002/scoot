use crate::commands::domain::Command;

/// Execute command (Tauri command)
#[tauri::command]
pub async fn execute_command(
    command: Command,
    args: Vec<String>,
    app_handle: tauri::AppHandle,
) -> Result<String, crate::error::AppError> {
    // Build command
    let final_command = if command.has_placeholders() {
        command.substitute_args(&args)
    } else {
        command.command.clone()
    };

    // Scoot command (check by source)
    if command.source == crate::commands::domain::SOURCE_SCOOT
        || final_command.starts_with("scoot://")
    {
        return execute_scoot_command(&app_handle, &final_command).await;
    }

    // Other commands - dispatch by category
    match command.category.as_str() {
        crate::commands::domain::CATEGORY_URL => execute_url(&app_handle, &final_command).await,
        crate::commands::domain::CATEGORY_FILE => {
            execute_local_file(&app_handle, &final_command).await
        }
        crate::commands::domain::CATEGORY_COMMAND => {
            execute_shell_command(
                &final_command,
                &command.working_dir,
                command.show_window.unwrap_or(false),
            )
            .await
        }
        // Default to shell command
        _ => {
            execute_shell_command(
                &final_command,
                &command.working_dir,
                command.show_window.unwrap_or(false),
            )
            .await
        }
    }
}

/// Execute internal scoot command
async fn execute_scoot_command(
    app_handle: &tauri::AppHandle,
    command: &str,
) -> Result<String, crate::error::AppError> {
    log::debug!("Executing scoot command: {}", command);

    match command {
        "scoot://add-command" => {
            crate::lifecycle::open_add_command_dialog(&app_handle)?;
            Ok("Opening add command dialog".to_string())
        }
        "scoot://open-commands" => {
            crate::commands::ipc::open_commands_json(app_handle.clone()).await?;
            Ok("Opened commands.json".to_string())
        }
        "scoot://open-config" => {
            crate::config::ipc::open_config_json(app_handle.clone()).await?;
            Ok("Opened config.json".to_string())
        }
        "scoot://open-readme" => {
            let resource_path = crate::os::resolve_resource(&app_handle, "README.md")?;
            crate::os::open_path(&app_handle, &resource_path.to_string_lossy())?;
            Ok("Opened README.md".to_string())
        }
        "scoot://open-log" => {
            crate::lifecycle::open_log(&app_handle)?;
            Ok("Log file opened".to_string())
        }
        "scoot://reload" => crate::lifecycle::reload(&app_handle)
            .await
            .map(|_| "Configuration and commands reloaded".to_string()),
        "scoot://kill" => {
            crate::system::quit_app((*app_handle).clone()).await?;
            Ok("Application terminated".to_string())
        }
        _ => Err(crate::error::AppError::CommandExecution(format!(
            "Unknown scoot command: {}",
            command
        ))),
    }
}

/// Execute URL command
async fn execute_url(
    app_handle: &tauri::AppHandle,
    url: &str,
) -> Result<String, crate::error::AppError> {
    log::debug!("Opening URL: {}", url);

    crate::os::open_url(app_handle, url).map_err(|e| {
        let error_msg = format!(
            "Failed to open URL '{}': {}. Please check if a web browser or associated application is installed.",
            url, e
        );
        log::error!("Error: {}", error_msg);
        crate::error::AppError::CommandExecution(error_msg)
    })?;

    let success_msg = format!("Successfully opened URL: {}", url);
    log::debug!("{}", success_msg);
    Ok(success_msg)
}

/// Execute local file
async fn execute_local_file(
    app_handle: &tauri::AppHandle,
    file_path: &str,
) -> Result<String, crate::error::AppError> {
    log::debug!("Opening file: {}", file_path);

    // Expand environment variables
    let expanded_path = crate::os::expand_env_vars(file_path);
    log::debug!("Opening file (expanded): {}", expanded_path);

    // Check if file exists
    if !std::path::Path::new(&expanded_path).exists() {
        let error_msg = format!(
            "File not found: '{}' (expanded from '{}'). Please check if the file exists and the path is correct.",
            expanded_path, file_path
        );
        log::error!("Error: {}", error_msg);
        return Err(crate::error::AppError::CommandExecution(error_msg));
    }

    // Open file
    crate::os::open_path(app_handle, &expanded_path).map_err(|e| {
        let error_msg = format!("Failed to open file '{}': {}. Please check file permissions and ensure a default application is set for this file type.", expanded_path, e);
        log::error!("Error: {}", error_msg);
        crate::error::AppError::CommandExecution(error_msg)
    })?;

    let success_msg = format!("Successfully opened file: {}", expanded_path);
    log::debug!("{}", success_msg);
    Ok(success_msg)
}

/// Execute a shell command
async fn execute_shell_command(
    command: &str,
    working_dir: &Option<String>,
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    show_window: bool,
) -> Result<String, crate::error::AppError> {
    use std::process::Command as StdCommand;
    log::debug!("Executing shell command: {}", command);

    // If the command is empty, return an error
    if command.trim().is_empty() {
        return Err(crate::error::AppError::Validation(
            "System command cannot be empty.".to_string(),
        ));
    }

    // Build the command
    #[cfg(target_os = "windows")]
    let mut cmd_builder = {
        use std::os::windows::process::CommandExt;
        let mut cmd = StdCommand::new("powershell");
        let mut args = vec!["-NoProfile"];
        if show_window {
            args.push("-NoExit");
        }
        args.push("-Command");
        args.push(command);
        cmd.args(args);
        if show_window {
            cmd.creation_flags(0x00000010); // CREATE_NEW_CONSOLE
        } else {
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        cmd
    };

    #[cfg(not(target_os = "windows"))]
    let mut cmd_builder = {
        let mut cmd = StdCommand::new("sh");
        cmd.args(["-c", command]);
        cmd
    };

    // Set the working directory
    if let Some(dir) = working_dir {
        // Remove double quotes if present
        let trimmed_dir = dir.trim();
        let clean_dir =
            if trimmed_dir.starts_with('"') && trimmed_dir.ends_with('"') && trimmed_dir.len() >= 2
            {
                &trimmed_dir[1..trimmed_dir.len() - 1]
            } else {
                trimmed_dir
            };

        if !clean_dir.is_empty() {
            if std::path::Path::new(clean_dir).exists() {
                cmd_builder.current_dir(clean_dir);
            } else {
                // If the working directory does not exist, issue a warning
                log::warn!(
                    "Warning: Working directory '{}' does not exist. Ignoring.",
                    clean_dir
                );
            }
        }
    }

    // Execute the command asynchronously
    match cmd_builder.spawn() {
        Ok(_) => {
            let success_msg = "Command launched successfully (background).".to_string();
            log::debug!("{}", success_msg);
            Ok(success_msg)
        }
        Err(e) => {
            let error_msg = format!("Failed to spawn system command '{}': {}.", command, e);
            log::error!("Error: {}", error_msg);
            Err(crate::error::AppError::CommandExecution(error_msg))
        }
    }
}
