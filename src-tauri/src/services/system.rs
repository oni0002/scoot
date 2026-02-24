use crate::store::state::AppState;
use tauri::{AppHandle, Manager, State};

/// Quit application
pub fn quit_app(app_handle: &AppHandle) {
    log::info!("Terminating application...");
    use tauri::Manager;

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
}

/// Reload config and commands
pub async fn reload(app_handle: &tauri::AppHandle) -> Result<(), crate::domain::error::AppError> {
    use tauri::Emitter;

    // Reload config (Config object is received)
    let config = if let Some(state) = app_handle.try_state::<AppState>() {
        let new_config =
            crate::services::config::reload(&state.config_manager, &state.config).await?;
        // Register hotkey again
        if let Err(e) =
            crate::services::shortcut::setup_global_shortcuts(app_handle, &new_config.hotkey)
        {
            log::warn!("Failed to re-register hotkey: {}", e);
        }
        new_config
    } else {
        return Err(crate::domain::error::AppError::System(
            "Failed to get AppState".to_string(),
        ));
    };

    // Reload commands (Config is passed)
    if let Some(state) = app_handle.try_state::<AppState>() {
        crate::services::command::reload(&state.config_manager, &state.commands, &config).await?;
    } else {
        return Err(crate::domain::error::AppError::System(
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
pub fn open_commands_json(app_handle: &AppHandle) -> Result<(), crate::domain::error::AppError> {
    if let Some(state) = app_handle.try_state::<AppState>() {
        let commands_path = state.config_manager.get_commands_path();

        crate::infra::system::ensure_file_exists(std::path::Path::new(&commands_path), || {
            let default_commands = crate::domain::command::Commands::default();
            tauri::async_runtime::block_on(async {
                state
                    .config_manager
                    .save_commands(&default_commands)
                    .await
                    .map_err(|e| crate::domain::error::AppError::System(e.to_string()))
            })
        })?;

        crate::infra::system::open_path(app_handle, &commands_path)?;
    }
    Ok(())
}

/// Open config.json
pub fn open_config_json(app_handle: &AppHandle) -> Result<(), crate::domain::error::AppError> {
    if let Some(state) = app_handle.try_state::<AppState>() {
        let config_path = state.config_manager.get_config_path();

        crate::infra::system::ensure_file_exists(std::path::Path::new(&config_path), || {
            let default_config = crate::domain::config::Config::default();
            tauri::async_runtime::block_on(async {
                state
                    .config_manager
                    .save(&default_config)
                    .await
                    .map_err(|e| crate::domain::error::AppError::System(e.to_string()))
            })
        })?;

        crate::infra::system::open_path(app_handle, &config_path)?;
    }
    Ok(())
}

/// Open README
pub fn open_readme(app_handle: &AppHandle) -> Result<(), crate::domain::error::AppError> {
    let resource_path = crate::infra::system::resolve_resource(app_handle, "README.md")?;
    let path_str = resource_path.to_string_lossy().to_string();
    crate::infra::system::open_path(app_handle, &path_str)
}

/// Get file watcher status
pub fn get_file_watcher_status(state: &State<'_, AppState>) -> bool {
    state._commands_file_watcher.is_some() || state._config_file_watcher.is_some()
}

/// Open log (or log directory)
pub fn open_log(app_handle: &AppHandle) -> Result<(), crate::domain::error::AppError> {
    let log_dir = crate::infra::system::get_log_dir(app_handle)?;
    crate::infra::system::ensure_directory_exists(&log_dir)?;

    let log_file = log_dir.join("scoot.log");
    let target_path = if log_file.exists() { log_file } else { log_dir };

    let path_str = target_path.to_string_lossy().to_string();
    log::debug!("Opening log path: {}", path_str);
    crate::infra::system::open_path(app_handle, &path_str)
}

/// Open add command dialog
pub fn open_add_command_dialog(
    app_handle: &AppHandle,
) -> Result<(), crate::domain::error::AppError> {
    use tauri::{Emitter, Manager};
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    app_handle.emit("open-add-command-dialog", ()).map_err(|e| {
        crate::domain::error::AppError::System(format!("Failed to emit add command event: {}", e))
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
                        crate::services::command::reload_bookmarks(&state.commands, &config).await
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
            if let Err(e) = crate::services::system::reload(&handle).await {
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
