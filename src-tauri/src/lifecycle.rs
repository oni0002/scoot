use crate::state::{CommandsState, SettingsState};
use tauri::{AppHandle, Manager, Runtime};

/// Reload settings and commands (IPC)
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

/// Reload settings and commands. A settings.json that fails to load does not stop the commands from
/// loading: the last good settings stays in force, the webview is told, and the error is returned
/// once everything else is done.
pub async fn reload<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<(), crate::error::AppError> {
    use tauri::Emitter;

    let Some(settings_state) = app_handle.try_state::<SettingsState>() else {
        return Err(crate::error::AppError::System(
            "Failed to get SettingsState".to_string(),
        ));
    };
    let (settings, settings_error) = match crate::settings::store::reload(
        &settings_state.settings_store,
        &settings_state.settings,
    )
    .await
    {
        Ok(new_settings) => {
            crate::notices::clear(app_handle, "settings");
            if let Err(e) =
                crate::shortcut::setup_global_shortcuts(app_handle, &new_settings.hotkey)
            {
                log::warn!("Failed to re-register hotkey: {}", e);
            }
            (new_settings, None)
        }
        Err(e) => {
            log::error!("Failed to load settings: {}", e);
            crate::notices::raise(app_handle, "settings", crate::notices::ERROR, e.to_string());
            let current = settings_state
                .settings
                .lock()
                .map_err(crate::error::AppError::lock)?
                .clone();
            (current, Some(e))
        }
    };

    if let Some(state) = app_handle.try_state::<CommandsState>() {
        crate::commands::loader::reload(
            app_handle,
            &state.command_store,
            &state.commands,
            &settings,
        )
        .await?;
    } else {
        return Err(crate::error::AppError::System(
            "Failed to get CommandsState".to_string(),
        ));
    }

    if let Err(e) = app_handle.emit("settings-reloaded", ()) {
        log::error!("Failed to emit settings-reloaded: {}", e);
    } else {
        log::info!("Reloaded settings and commands");
    }

    settings_error.map_or(Ok(()), Err)
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

/// How long reload events must stay quiet before the reload runs. An editor saves in several
/// steps, and the app's own save is followed by a file event; the reload should read the finished
/// file, once.
const RELOAD_QUIET_PERIOD: std::time::Duration = std::time::Duration::from_millis(300);

/// Event for a reload that must happen (the sources have to be scanned again)
pub const REQUEST_RELOAD: &str = "request-reload";
/// Event for a change seen in settings.json. It reloads only when the file differs from the
/// settings in memory: the app's own saves already updated them.
pub const SETTINGS_FILE_CHANGED: &str = "settings-file-changed";

/// Run `action` once the `event` has been quiet for `RELOAD_QUIET_PERIOD`
fn on_quiet<F, Fut>(app: &tauri::App, event: &'static str, action: F)
where
    F: Fn(tauri::AppHandle) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tauri::Listener;

    let app_handle = app.handle().clone();
    let latest_request = Arc::new(AtomicU64::new(0));
    let action = Arc::new(action);

    app.listen(event, move |_event| {
        log::debug!("Received {} event", event);
        let this_request = latest_request.fetch_add(1, Ordering::SeqCst) + 1;
        let latest_request = latest_request.clone();
        let handle = app_handle.clone();
        let action = action.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(RELOAD_QUIET_PERIOD).await;
            // A newer request is waiting; it will do the work
            if latest_request.load(Ordering::SeqCst) != this_request {
                return;
            }
            action(handle).await;
        });
    });
}

/// Setup reload event listeners
pub fn setup_reload_listeners(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    on_quiet(app, REQUEST_RELOAD, |handle| async move {
        if let Err(e) = reload(&handle).await {
            log::error!("Failed to reload configuration: {}", e);
        }
    });
    on_quiet(app, SETTINGS_FILE_CHANGED, |handle| async move {
        if let Err(e) = reload_if_settings_differ(&handle).await {
            log::error!("Failed to reload configuration: {}", e);
        }
    });
    Ok(())
}

/// Reload unless settings.json holds what is already in memory, as it does after the app's own
/// save. Rescanning bookmarks and applications for that would be wasted work.
pub async fn reload_if_settings_differ<R: Runtime>(
    app_handle: &AppHandle<R>,
) -> Result<(), crate::error::AppError> {
    if let Some(state) = app_handle.try_state::<SettingsState>() {
        if let Ok(on_disk) = state.settings_store.load().await {
            let unchanged = state
                .settings
                .lock()
                .map_err(crate::error::AppError::lock)?
                .eq(&on_disk);
            if unchanged {
                // A file fixed back to the last good settings still ends the error it raised
                crate::notices::clear(app_handle, "settings");
                return Ok(());
            }
        }
    }
    reload(app_handle).await
}

/// How often the periodic reload checks whether its interval has passed. The interval is read
/// each time, so a change in Settings takes effect within this much.
const PERIODIC_CHECK: std::time::Duration = std::time::Duration::from_secs(60);

/// Start background task for periodic global reload
pub fn start_periodic_reload(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last_reload = std::time::Instant::now();
        loop {
            tokio::time::sleep(PERIODIC_CHECK).await;

            let interval_minutes = app_handle
                .try_state::<SettingsState>()
                .and_then(|state| {
                    state
                        .settings
                        .lock()
                        .ok()
                        .map(|s| s.reload_interval_minutes)
                })
                .unwrap_or(30);
            if last_reload.elapsed() < std::time::Duration::from_secs(interval_minutes * 60) {
                continue;
            }
            last_reload = std::time::Instant::now();

            log::debug!("Executing scheduled global reload...");

            if let Err(e) = reload(&app_handle).await {
                log::error!("Failed to auto-reload system: {}", e);
            }
        }
    });
}
