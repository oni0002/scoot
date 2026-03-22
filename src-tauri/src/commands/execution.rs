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

    // Scoot command
    if command.category == crate::commands::domain::CATEGORY_SCOOT
        || final_command.starts_with("scoot://")
    {
        return execute_scoot_command(&app_handle, &final_command).await;
    }

    // Other commands
    match command.category.as_str() {
        // URL, bookmark
        crate::commands::domain::CATEGORY_URL | crate::commands::domain::CATEGORY_BOOKMARK => {
            execute_url(&app_handle, &final_command).await
        }
        // File, application
        crate::commands::domain::CATEGORY_FILE | crate::commands::domain::CATEGORY_APPLICATION => {
            execute_local_file(&app_handle, &final_command).await
        }
        // Shell command
        crate::commands::domain::CATEGORY_COMMAND => {
            crate::system::execute_shell_command(
                &final_command,
                &command.working_dir,
                command.show_window.unwrap_or(false),
            )
            .await
        }
        // Default to shell command
        _ => {
            crate::system::execute_shell_command(
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
            crate::system::open_add_command_dialog(&app_handle)?;
            Ok("Opening add command dialog".to_string())
        }
        "scoot://open-commands" => {
            crate::system::open_commands_json(&app_handle)?;
            Ok("Opened commands.json".to_string())
        }
        "scoot://open-config" => {
            crate::system::open_config_json(&app_handle)?;
            Ok("Opened config.json".to_string())
        }
        "scoot://open-readme" => {
            crate::system::open_readme(app_handle.clone()).await?;
            Ok("Opened README.md".to_string())
        }
        "scoot://open-log" => {
            crate::system::open_log(&app_handle)?;
            Ok("Log file opened".to_string())
        }
        "scoot://reload" => crate::system::reload(&app_handle)
            .await
            .map(|_| "Configuration and commands reloaded".to_string()),
        "scoot://kill" => {
            crate::system::quit_app_command(app_handle.clone()).await?;
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

    crate::system::open_url(app_handle, url).map_err(|e| {
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
    let expanded_path = crate::system::expand_env_vars(file_path);
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
    crate::system::open_path(app_handle, &expanded_path).map_err(|e| {
        let error_msg = format!("Failed to open file '{}': {}. Please check file permissions and ensure a default application is set for this file type.", expanded_path, e);
        log::error!("Error: {}", error_msg);
        crate::error::AppError::CommandExecution(error_msg)
    })?;

    let success_msg = format!("Successfully opened file: {}", expanded_path);
    log::debug!("{}", success_msg);
    Ok(success_msg)
}
