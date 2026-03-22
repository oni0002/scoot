use crate::state::AppState;
use regex::Regex;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

// --- Tauri Commands ---

/// Reload config
#[tauri::command]
pub async fn reload_config(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    reload(&app_handle).await
}

/// Get file watcher status
#[tauri::command]
pub async fn get_file_watcher_status(
    state: State<'_, AppState>,
) -> Result<bool, crate::error::AppError> {
    Ok(state._commands_file_watcher.is_some() || state._config_file_watcher.is_some())
}

/// Open README
#[tauri::command]
pub async fn open_readme(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    let resource_path = resolve_resource(&app_handle, "README.md")?;
    let path_str = resource_path.to_string_lossy().to_string();
    open_path(&app_handle, &path_str)
}

/// Quit app
#[tauri::command]
pub async fn quit_app_command(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    log::info!("Terminating application...");

    // Hide windows (for user responsiveness)
    for window in app_handle.webview_windows().values() {
        let _ = window.hide();
    }

    // Unregister global shortcuts
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let _ = app_handle.global_shortcut().unregister_all();

    // Try to save settings
    if let Some(state) = app_handle.try_state::<AppState>() {
        // try_lock (deadlock avoidance)
        if let Ok(manager) = state.commands.try_lock() {
            let commands_config = manager.get_user_commands();
            let _ = tauri::async_runtime::block_on(async {
                state.config_manager.save_commands(&commands_config).await
            });
        }
    }

    app_handle.exit(0);
    Ok(())
}

/// Reload config and commands
pub async fn reload(app_handle: &tauri::AppHandle) -> Result<(), crate::error::AppError> {
    use tauri::Emitter;

    // Reload config (Config object is received)
    let config = if let Some(state) = app_handle.try_state::<AppState>() {
        let new_config = crate::config::store::reload(&state.config_manager, &state.config).await?;
        // Register hotkey again
        if let Err(e) = crate::shortcut::setup_global_shortcuts(app_handle, &new_config.hotkey) {
            log::warn!("Failed to re-register hotkey: {}", e);
        }
        new_config
    } else {
        return Err(crate::error::AppError::System(
            "Failed to get AppState".to_string(),
        ));
    };

    // Reload commands (Config is passed)
    if let Some(state) = app_handle.try_state::<AppState>() {
        crate::commands::ipc::reload(&state.config_manager, &state.commands, &config).await?;
    } else {
        return Err(crate::error::AppError::System(
            "Failed to get AppState".to_string(),
        ));
    }

    if let Err(e) = app_handle.emit("config-reloaded", ()) {
        log::error!("Failed to emit config-reloaded: {}", e);
    } else {
        log::info!("Config reloaded successfully, event emitted.");
    }

    Ok(())
}

/// Open commands.json
pub fn open_commands_json(app_handle: &AppHandle) -> Result<(), crate::error::AppError> {
    if let Some(state) = app_handle.try_state::<AppState>() {
        let commands_path = state.config_manager.get_commands_path();

        ensure_file_exists(std::path::Path::new(&commands_path), || {
            let default_commands = crate::commands::domain::Commands::default();
            tauri::async_runtime::block_on(async {
                state
                    .config_manager
                    .save_commands(&default_commands)
                    .await
                    .map_err(|e| crate::error::AppError::System(e.to_string()))
            })
        })?;

        open_path(app_handle, &commands_path)?;
    }
    Ok(())
}

/// Open config.json
pub fn open_config_json(app_handle: &AppHandle) -> Result<(), crate::error::AppError> {
    if let Some(state) = app_handle.try_state::<AppState>() {
        let config_path = state.config_manager.get_config_path();

        ensure_file_exists(std::path::Path::new(&config_path), || {
            let default_config = crate::config::domain::Config::default();
            tauri::async_runtime::block_on(async {
                state
                    .config_manager
                    .save(&default_config)
                    .await
                    .map_err(|e| crate::error::AppError::System(e.to_string()))
            })
        })?;

        open_path(app_handle, &config_path)?;
    }
    Ok(())
}

/// Open log (or log directory)
pub fn open_log(app_handle: &AppHandle) -> Result<(), crate::error::AppError> {
    let log_dir = get_log_dir(app_handle)?;
    ensure_directory_exists(&log_dir)?;

    let log_file = log_dir.join("scoot.log");
    let target_path = if log_file.exists() { log_file } else { log_dir };

    let path_str = target_path.to_string_lossy().to_string();
    log::debug!("Opening log path: {}", path_str);
    open_path(app_handle, &path_str)
}

/// Open add command dialog
pub fn open_add_command_dialog(app_handle: &AppHandle) -> Result<(), crate::error::AppError> {
    use tauri::{Emitter, Manager};
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    app_handle.emit("open-add-command-dialog", ()).map_err(|e| {
        crate::error::AppError::System(format!("Failed to emit add command event: {}", e))
    })
}

/// Start bookmark auto-refresh task
pub fn start_bookmark_update_task(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        log::debug!("Starting bookmark auto-refresh task");
        loop {
            // Get refresh interval from current config
            let interval_minutes = if let Some(state) = app_handle.try_state::<AppState>() {
                if let Ok(config) = state.config.lock() {
                    // Minimum value limit (1 minute)
                    std::cmp::max(config.bookmarks.refresh_interval_minutes, 1)
                } else {
                    30 // Lock acquisition failure
                }
            } else {
                30 // State acquisition failure
            };

            // Wait for specified time
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_minutes * 60)).await;

            log::debug!("Executing scheduled bookmark refresh...");

            // Get config and reload
            let config_opt = if let Some(state) = app_handle.try_state::<AppState>() {
                state.config.lock().ok().map(|c| c.clone())
            } else {
                None
            };

            if let Some(config) = config_opt {
                // Reload bookmarks only
                if let Some(state) = app_handle.try_state::<AppState>() {
                    if let Err(e) =
                        crate::commands::ipc::reload_bookmarks(&state.commands, &config).await
                    {
                        log::error!("Failed to auto-refresh bookmarks: {}", e);
                    }
                }
            } else {
                log::warn!("Skipping bookmark refresh due to failure in retrieving AppState or Config lock");
            }
        }
    });
}

/// Setup event listeners
pub fn setup_event_listeners(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    use tauri::{Emitter, Listener};

    let app_handle = app.handle().clone();
    let last_reload = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(1)));

    // Listen for config reload event
    app.listen("request-reload", move |_event| {
        log::debug!("Received request-reload event");
        if let Ok(mut last) = last_reload.lock() {
            // Ignore if less than 500ms (debounce)
            if last.elapsed() < Duration::from_millis(500) {
                return;
            }
            *last = Instant::now();
        }

        // Asynchronously reload config
        let handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = reload(&handle).await {
                log::error!("Failed to reload configuration: {}", e);
            }
        });
    });

    // Proxy file change events to reload request
    let app_handle_for_proxy = app.handle().clone();
    app.listen("config-file-changed", move |_| {
        let _ = app_handle_for_proxy.emit("request-reload", ());
    });

    Ok(())
}

/// Open the specified path (file, directory, URL) with the default application
pub fn open_path(app_handle: &AppHandle, path: &str) -> Result<(), crate::error::AppError> {
    app_handle
        .opener()
        .open_path(path, None::<&str>)
        .map_err(|e| {
            crate::error::AppError::System(format!("Failed to open path '{}': {}", path, e))
        })
}

/// Open the specified URL with the default browser
pub fn open_url(app_handle: &AppHandle, url: &str) -> Result<(), crate::error::AppError> {
    app_handle
        .opener()
        .open_url(url, None::<&str>)
        .map_err(|e| crate::error::AppError::System(format!("Failed to open URL '{}': {}", url, e)))
}

/// Ensure the file exists, create it with default content if not
pub fn ensure_file_exists<F>(path: &Path, create_content: F) -> Result<(), crate::error::AppError>
where
    F: FnOnce() -> Result<(), crate::error::AppError>,
{
    if !path.exists() {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| crate::error::AppError::System(e.to_string()))?;
            }
        }
        create_content()?;
    }
    Ok(())
}

/// Ensure the directory exists, create it if not
pub fn ensure_directory_exists(path: &Path) -> Result<(), crate::error::AppError> {
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| crate::error::AppError::System(e.to_string()))?;
    }
    Ok(())
}

/// Resolve the path to a resource file
pub fn resolve_resource(
    app_handle: &AppHandle,
    path: &str,
) -> Result<PathBuf, crate::error::AppError> {
    let resource_path = app_handle
        .path()
        .resolve(path, tauri::path::BaseDirectory::Resource)
        .map_err(|e| {
            crate::error::AppError::System(format!(
                "Failed to resolve resource path '{}': {}",
                path, e
            ))
        })?;

    if !resource_path.exists() {
        return Err(crate::error::AppError::NotFound(format!(
            "Resource not found: {}",
            path
        )));
    }

    Ok(resource_path)
}

/// Get the path to the log directory
pub fn get_log_dir(_app_handle: &AppHandle) -> Result<PathBuf, crate::error::AppError> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("logs")))
        .ok_or_else(|| {
            crate::error::AppError::System("Failed to determine log directory".to_string())
        })
}

/// Execute a shell command
pub async fn execute_shell_command(
    command: &str,
    working_dir: &Option<String>,
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
            Err(crate::error::AppError::CommandExecution(error_msg))
        }
    }
}

/// Expand Windows environment variables like %APPDATA%
pub fn expand_env_vars(path: &str) -> String {
    let re = Regex::new(r"%([^%]+)%").unwrap();

    re.replace_all(path, |caps: &regex::Captures| {
        let key = &caps[1];
        std::env::var(key).unwrap_or_else(|_| format!("%{}%", key))
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_expand_existing_var() {
        let key = "SCOOT_TEST_VAR";
        let value = "test_value";
        env::set_var(key, value);

        let input = format!("path/to/%{}%/file", key);
        let expected = format!("path/to/{}/file", value);

        assert_eq!(expand_env_vars(&input), expected);

        env::remove_var(key);
    }

    #[test]
    fn test_expand_non_existing_var() {
        let input = "path/to/%NON_EXISTING_VAR%/file";
        assert_eq!(expand_env_vars(input), input);
    }

    #[test]
    fn test_no_vars() {
        let input = "path/to/file";
        assert_eq!(expand_env_vars(input), input);
    }
}
