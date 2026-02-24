use crate::store::state::AppState;
use tauri::{AppHandle, Emitter, Manager};

/// Show the window
pub fn show(app_handle: &AppHandle) -> Result<(), crate::domain::error::AppError> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("window-shown", ());

        // Record the time the window was shown
        if let Some(state) = app_handle.try_state::<AppState>() {
            if let Ok(mut last_shown) = state.last_window_shown.lock() {
                *last_shown = Some(std::time::Instant::now());
            }
        }
    }
    Ok(())
}

/// Hide the window
pub fn hide(app_handle: &AppHandle) -> Result<(), crate::domain::error::AppError> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}
