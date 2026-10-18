use crate::state::{CommandsState, ConfigState};
use tauri::{AppHandle, Manager};

/// Reload config and commands (IPC)
#[tauri::command]
pub async fn reload_all(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    reload(&app_handle).await
}

/// Quit app. commands.json needs no final write: every change is saved as it happens.
pub async fn quit_app(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    log::info!("Terminating application...");

    for window in app_handle.webview_windows().values() {
        let _ = window.hide();
    }

    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let _ = app_handle.global_shortcut().unregister_all();

    app_handle.exit(0);
    Ok(())
}

/// Reload config and commands. A config.json that fails to load does not stop the commands from
/// loading: the last good config stays in force, the webview is told, and the error is returned
/// once everything else is done.
pub async fn reload(app_handle: &tauri::AppHandle) -> Result<(), crate::error::AppError> {
    use tauri::Emitter;

    let Some(config_state) = app_handle.try_state::<ConfigState>() else {
        return Err(crate::error::AppError::System(
            "Failed to get ConfigState".to_string(),
        ));
    };
    let (config, config_error) = match crate::config::store::reload(
        &config_state.config_store,
        &config_state.config,
    )
    .await
    {
        Ok(new_config) => {
            crate::notices::clear(app_handle, "config");
            if let Err(e) = crate::shortcut::setup_global_shortcuts(app_handle, &new_config.hotkey)
            {
                log::warn!("Failed to re-register hotkey: {}", e);
            }
            (new_config, None)
        }
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            crate::notices::raise(app_handle, "config", crate::notices::ERROR, e.to_string());
            let current = config_state
                .config
                .lock()
                .map_err(crate::error::AppError::lock)?
                .clone();
            (current, Some(e))
        }
    };

    if let Some(state) = app_handle.try_state::<CommandsState>() {
        crate::commands::loader::reload(app_handle, &state.command_store, &state.commands, &config)
            .await?;
    } else {
        return Err(crate::error::AppError::System(
            "Failed to get CommandsState".to_string(),
        ));
    }

    if let Err(e) = app_handle.emit("config-reloaded", ()) {
        log::error!("Failed to emit config-reloaded: {}", e);
    } else {
        log::info!("Reloaded config and commands");
    }

    config_error.map_or(Ok(()), Err)
}

/// Open log (or log directory)
pub fn open_log(app_handle: &AppHandle) -> Result<(), crate::error::AppError> {
    let log_dir = crate::os::log_dir();
    std::fs::create_dir_all(&log_dir)?;

    let log_file = log_dir.join("scoot.log");
    let target_path = if log_file.exists() { log_file } else { log_dir };

    let path_str = target_path.to_string_lossy().to_string();
    log::debug!("Opening log path: {}", path_str);
    crate::os::open_path(app_handle, &path_str)
}

/// Open add command dialog
pub fn open_add_command_dialog(app_handle: &AppHandle) -> Result<(), crate::error::AppError> {
    show_and_emit(app_handle, "open-add-command-dialog")
}

/// Open settings screen
pub fn open_settings_dialog(app_handle: &AppHandle) -> Result<(), crate::error::AppError> {
    show_and_emit(app_handle, "open-settings-dialog")
}

/// Bring the main window forward and tell the webview which screen to open
fn show_and_emit(app_handle: &AppHandle, event: &str) -> Result<(), crate::error::AppError> {
    use tauri::Emitter;
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    app_handle
        .emit(event, ())
        .map_err(|e| crate::error::AppError::System(format!("Failed to emit {}: {}", event, e)))
}

/// How long `request-reload` events must stay quiet before the reload runs. An editor saves in
/// several steps, and the app's own save is followed by a file event; the reload should read the
/// finished file, once.
const RELOAD_QUIET_PERIOD: std::time::Duration = std::time::Duration::from_millis(300);

/// Setup reload event listeners
pub fn setup_reload_listeners(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tauri::Listener;

    let app_handle = app.handle().clone();
    let latest_request = Arc::new(AtomicU64::new(0));

    app.listen("request-reload", move |_event| {
        log::debug!("Received request-reload event");
        let this_request = latest_request.fetch_add(1, Ordering::SeqCst) + 1;
        let latest_request = latest_request.clone();
        let handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(RELOAD_QUIET_PERIOD).await;
            // A newer request is waiting; it will do the reload
            if latest_request.load(Ordering::SeqCst) != this_request {
                return;
            }
            if let Err(e) = reload(&handle).await {
                log::error!("Failed to reload configuration: {}", e);
            }
        });
    });

    Ok(())
}

/// Start background task for periodic global reload
pub fn start_periodic_reload(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let interval_minutes = if let Some(state) = app_handle.try_state::<ConfigState>() {
                if let Ok(config) = state.config.lock() {
                    config.reload_interval_minutes
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
