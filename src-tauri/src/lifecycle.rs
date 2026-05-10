use crate::state::AppState;
use tauri::{AppHandle, Manager};

/// Reload config and commands
pub async fn reload(app_handle: &tauri::AppHandle) -> Result<(), crate::error::AppError> {
    use tauri::Emitter;

    let config = if let Some(state) = app_handle.try_state::<AppState>() {
        let new_config = crate::config::store::reload(&state.config_store, &state.config).await?;
        if let Err(e) = crate::shortcut::setup_global_shortcuts(app_handle, &new_config.hotkey) {
            log::warn!("Failed to re-register hotkey: {}", e);
        }
        new_config
    } else {
        return Err(crate::error::AppError::System(
            "Failed to get AppState".to_string(),
        ));
    };

    if let Some(state) = app_handle.try_state::<AppState>() {
        crate::commands::loader::reload(&state.command_store, &state.commands, &config).await?;
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
    let log_dir = crate::os::get_log_dir(app_handle)?;
    std::fs::create_dir_all(&log_dir)?;

    let log_file = log_dir.join("scoot.log");
    let target_path = if log_file.exists() { log_file } else { log_dir };

    let path_str = target_path.to_string_lossy().to_string();
    log::debug!("Opening log path: {}", path_str);
    crate::os::open_path(app_handle, &path_str)
}

/// Open add command dialog
pub fn open_add_command_dialog(app_handle: &AppHandle) -> Result<(), crate::error::AppError> {
    use tauri::Emitter;
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

    app.listen("request-reload", move |_event| {
        log::debug!("Received request-reload event");
        if let Ok(mut last) = last_reload.lock() {
            if last.elapsed() < Duration::from_millis(500) {
                return;
            }
            *last = Instant::now();
        }

        let handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = reload(&handle).await {
                log::error!("Failed to reload configuration: {}", e);
            }
        });
    });

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
            let interval_minutes = if let Some(state) = app_handle.try_state::<AppState>() {
                if let Ok(config) = state.config.lock() {
                    std::cmp::max(config.reload_interval_minutes, 1)
                } else {
                    30
                }
            } else {
                30
            };

            tokio::time::sleep(std::time::Duration::from_secs(interval_minutes * 60)).await;

            log::debug!("Executing scheduled global reload...");

            if let Err(e) = reload(&app_handle).await {
                log::error!("Failed to auto-reload system: {}", e);
            }
        }
    });
}
