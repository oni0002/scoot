use crate::state::AppState;
use tauri::{Emitter, Manager, State};

// --- Tauri Commands ---

/// Toggle window visibility
#[tauri::command]
pub async fn toggle_window(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::error::AppError> {
    if let Some(window) = app_handle.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
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
    }
    Ok(())
}

/// Hide window
#[tauri::command]
pub async fn hide_window(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::error::AppError> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}

/// Show window
#[tauri::command]
pub async fn show_window(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::error::AppError> {
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

/// Set prevent_hide flag
#[tauri::command]
pub async fn set_prevent_hide(
    prevent: bool,
    state: State<'_, AppState>,
) -> Result<(), crate::error::AppError> {
    set_prevent_hide_flag(&state.prevent_hide, prevent)
}

// --- Infrastructure / Core Logic ---



/// Set prevent_hide flag
pub fn set_prevent_hide_flag(
    prevent_hide_mutex: &std::sync::Mutex<bool>,
    prevent: bool,
) -> Result<(), crate::error::AppError> {
    if let Ok(mut flag) = prevent_hide_mutex.lock() {
        *flag = prevent;
        Ok(())
    } else {
        Err(crate::error::AppError::System(
            "Failed to lock prevent_hide flag".to_string(),
        ))
    }
}

/// Handle focus change event
pub fn handle_focus_change(
    window: &tauri::WebviewWindow,
    focused: bool,
    last_window_shown: &std::sync::Mutex<Option<std::time::Instant>>,
    prevent_hide: &std::sync::Mutex<bool>,
    last_window_hidden: &std::sync::Mutex<Option<std::time::Instant>>,
) {
    // If focused, do nothing
    if focused {
        return;
    }

    // If shown within 200ms, do nothing
    if let Ok(last_shown) = last_window_shown.lock() {
        if let Some(ref instant) = *last_shown {
            if instant.elapsed().as_millis() <= 200 {
                return;
            }
        }
    }

    // If prevent_hide is true, do nothing
    if let Ok(flag) = prevent_hide.lock() {
        if *flag {
            return;
        }
    }

    // Hide window
    if let Ok(mut hidden) = last_window_hidden.lock() {
        *hidden = Some(std::time::Instant::now());
    }
    let _ = window.hide();
}

/// Setup window events
pub fn setup_window_events(
    app: &tauri::App,
    last_window_shown: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
    prevent_hide: std::sync::Arc<std::sync::Mutex<bool>>,
    last_window_hidden: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
) {
    if let Some(window) = app.get_webview_window("main") {
        let window_clone = window.clone();
        window.on_window_event(move |event| {
            match event {
                // When window close request is sent
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window_clone.hide();
                }
                // When focus changes
                tauri::WindowEvent::Focused(focused) => {
                    handle_focus_change(
                        &window_clone,
                        *focused,
                        &last_window_shown,
                        &prevent_hide,
                        &last_window_hidden,
                    );
                }
                _ => {}
            }
        });
        // Hide window on startup
        let _ = window.hide();
    }
}
