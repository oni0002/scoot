use crate::domain::config::DEFAULT_SHORTCUT;
use crate::store::state::AppState;
use tauri::{App, Manager};

/// Setup shortcuts
pub fn setup_shortcuts(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle();
    // Get hotkey from state
    let hotkey = if let Some(state) = handle.try_state::<AppState>() {
        if let Ok(config) = state.config.lock() {
            config.hotkey.clone()
        } else {
            DEFAULT_SHORTCUT.to_string()
        }
    } else {
        DEFAULT_SHORTCUT.to_string()
    };

    setup_global_shortcuts(handle, &hotkey)
}

/// Register global shortcuts
pub fn setup_global_shortcuts(
    app_handle: &tauri::AppHandle,
    hotkey: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::infra::shortcut::register(app_handle, hotkey, move |handle| {
        // Callback: toggle window visibility
        if let Err(e) = crate::services::window::toggle_visibility(handle) {
            log::error!("Failed to toggle window visibility via shortcut: {}", e);
        }
    })
}
