use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

/// Open the specified path (file, directory, URL) with the default application
pub fn open_path(app_handle: &AppHandle, path: &str) -> Result<(), crate::domain::error::AppError> {
    app_handle
        .opener()
        .open_path(path, None::<&str>)
        .map_err(|e| {
            crate::domain::error::AppError::System(format!("Failed to open path '{}': {}", path, e))
        })
}

/// Open the specified URL with the default browser
pub fn open_url(app_handle: &AppHandle, url: &str) -> Result<(), crate::domain::error::AppError> {
    app_handle
        .opener()
        .open_url(url, None::<&str>)
        .map_err(|e| {
            crate::domain::error::AppError::System(format!("Failed to open URL '{}': {}", url, e))
        })
}

/// Ensure the file exists, create it with default content if not
pub fn ensure_file_exists<F>(
    path: &Path,
    create_content: F,
) -> Result<(), crate::domain::error::AppError>
where
    F: FnOnce() -> Result<(), crate::domain::error::AppError>,
{
    if !path.exists() {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
            }
        }
        create_content()?;
    }
    Ok(())
}

/// Ensure the directory exists, create it if not
pub fn ensure_directory_exists(path: &Path) -> Result<(), crate::domain::error::AppError> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
    }
    Ok(())
}

/// Resolve the path to a resource file
pub fn resolve_resource(
    app_handle: &AppHandle,
    path: &str,
) -> Result<PathBuf, crate::domain::error::AppError> {
    let resource_path = app_handle
        .path()
        .resolve(path, tauri::path::BaseDirectory::Resource)
        .map_err(|e| {
            crate::domain::error::AppError::System(format!(
                "Failed to resolve resource path '{}': {}",
                path, e
            ))
        })?;

    if !resource_path.exists() {
        return Err(crate::domain::error::AppError::NotFound(format!(
            "Resource not found: {}",
            path
        )));
    }

    Ok(resource_path)
}

/// Get the path to the log directory
pub fn get_log_dir(_app_handle: &AppHandle) -> Result<PathBuf, crate::domain::error::AppError> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("logs")))
        .ok_or_else(|| {
            crate::domain::error::AppError::System("Failed to determine log directory".to_string())
        })
}

/// Execute a shell command
pub async fn execute_shell_command(
    command: &str,
    working_dir: &Option<String>,
    show_window: bool,
) -> Result<String, crate::domain::error::AppError> {
    use std::process::Command as StdCommand;
    log::debug!("Executing shell command: {}", command);

    // If the command is empty, return an error
    if command.trim().is_empty() {
        return Err(crate::domain::error::AppError::Validation(
            "System command cannot be empty.".to_string(),
        ));
    }

    // Build the command
    let mut cmd_builder = if cfg!(target_os = "windows") {
        use std::os::windows::process::CommandExt;
        // PowerShell to call directly
        let mut cmd = StdCommand::new("powershell");
        // Skip profile loading for faster execution
        let mut args = vec!["-NoProfile"];
        // If show_window is true, add -NoExit (to prevent window closure)
        if show_window {
            args.push("-NoExit");
        }

        args.push("-Command");
        args.push(command);

        cmd.args(args);

        if show_window {
            // Create a new console window (CREATE_NEW_CONSOLE)
            cmd.creation_flags(0x00000010);
        } else {
            // Do not show the window (CREATE_NO_WINDOW)
            cmd.creation_flags(0x08000000);
        }
        cmd
    } else {
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
            Err(crate::domain::error::AppError::CommandExecution(error_msg))
        }
    }
}
