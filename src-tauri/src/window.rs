use crate::state::AppState;
use tauri::{AppHandle, Emitter, Manager, State};

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("window-shown", ());

        if let Some(state) = app.try_state::<AppState>() {
            if let Ok(mut last_shown) = state.last_window_shown.lock() {
                *last_shown = Some(std::time::Instant::now());
            }
        }
    }
}

// --- Tauri Commands ---

/// Toggle window visibility
pub async fn toggle_window(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    if let Some(window) = app_handle.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            show_main_window(&app_handle);
        }
    }
    Ok(())
}

/// Hide window
#[tauri::command]
pub async fn hide_window(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}

/// Show window
pub async fn show_window(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    show_main_window(&app_handle);
    Ok(())
}

/// Increment the modal refcount to prevent auto-hide
#[tauri::command]
pub async fn enter_modal(state: State<'_, AppState>) -> Result<(), crate::error::AppError> {
    state.prevent_hide.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

/// Decrement the modal refcount; auto-hide resumes when count reaches zero
#[tauri::command]
pub async fn leave_modal(state: State<'_, AppState>) -> Result<(), crate::error::AppError> {
    let prev = state.prevent_hide.load(std::sync::atomic::Ordering::SeqCst);
    if prev > 0 {
        state.prevent_hide.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
    Ok(())
}

/// Handle focus change event
pub fn handle_focus_change(
    window: &tauri::WebviewWindow,
    focused: bool,
    last_window_shown: &std::sync::Mutex<Option<std::time::Instant>>,
    prevent_hide: &std::sync::atomic::AtomicUsize,
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

    // If any modal is open, do nothing
    if prevent_hide.load(std::sync::atomic::Ordering::SeqCst) > 0 {
        return;
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
    prevent_hide: std::sync::Arc<std::sync::atomic::AtomicUsize>,
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
