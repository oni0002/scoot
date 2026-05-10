use crate::state::AppState;
use regex::Regex;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

/// Reload config and commands
#[tauri::command]
pub async fn reload_all(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    reload(&app_handle).await
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
pub async fn quit_app(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
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
        let commands_config_opt = {
            if let Ok(manager) = state.commands.try_lock() {
                Some(manager.get_user_commands())
            } else {
                None
            }
        };

        if let Some(commands_config) = commands_config_opt {
            let _ = state.command_store.save(&commands_config).await;
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
        let new_config = crate::config::store::reload(&state.config_store, &state.config).await?;
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
        crate::commands::ipc::reload(&state.command_store, &state.commands, &config).await?;
    } else {
        return Err(crate::error::AppError::System(
            "Failed to get AppState".to_string(),
        ));
    }

    if let Err(e) = app_handle.emit("config-reloaded", ()) {
        log::error!("Failed to emit config-reloaded: {}", e);
    } else {
        log::info!("Reloaded config and commands");
    }

    Ok(())
}

/// Open log (or log directory)
pub fn open_log(app_handle: &AppHandle) -> Result<(), crate::error::AppError> {
    let log_dir = get_log_dir(app_handle)?;
    std::fs::create_dir_all(&log_dir)?;

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

/// Setup reload event listeners
pub fn setup_reload_listeners(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
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

/// Start background task for periodic global reload
pub fn start_periodic_reload(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            // Get reload interval from config
            let interval_minutes = if let Some(state) = app_handle.try_state::<AppState>() {
                if let Ok(config) = state.config.lock() {
                    std::cmp::max(config.reload_interval_minutes, 1)
                } else {
                    30 // Lock failure
                }
            } else {
                30 // State failure
            };

            // Wait for specified time
            tokio::time::sleep(std::time::Duration::from_secs(interval_minutes * 60)).await;

            log::debug!("Executing scheduled global reload...");

            // Perform full reload
            if let Err(e) = reload(&app_handle).await {
                log::error!("Failed to auto-reload system: {}", e);
            }
        }
    });
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
