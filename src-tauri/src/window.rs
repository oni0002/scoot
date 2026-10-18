use crate::state::WindowState;
use tauri::{AppHandle, Emitter, Manager, State};

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("window-shown", ());

        if let Some(state) = app.try_state::<WindowState>() {
            if let Ok(mut last_shown) = state.last_window_shown.lock() {
                *last_shown = Some(std::time::Instant::now());
            }
        }
    }
}

pub fn is_main_window_visible(app: &AppHandle) -> bool {
    app.get_webview_window("main")
        .is_some_and(|w| w.is_visible().unwrap_or(false))
}

/// True while a dialog or secondary screen holds the window open
pub fn is_modal_open(app: &AppHandle) -> bool {
    app.try_state::<WindowState>()
        .is_some_and(|s| s.prevent_hide.load(std::sync::atomic::Ordering::SeqCst) > 0)
}

/// Hide the main window if it is visible, show it otherwise
pub fn toggle_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        show_main_window(app);
    }
}

// --- Tauri Commands ---

/// Hide window
#[tauri::command]
pub async fn hide_window(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}

/// Increment the modal refcount to prevent auto-hide
#[tauri::command]
pub async fn enter_modal(state: State<'_, WindowState>) -> Result<(), crate::error::AppError> {
    state
        .prevent_hide
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

/// Decrement the modal refcount; auto-hide resumes when count reaches zero
#[tauri::command]
pub async fn leave_modal(state: State<'_, WindowState>) -> Result<(), crate::error::AppError> {
    release_modal(&state.prevent_hide);
    Ok(())
}

/// Lower the count by one but never below zero. One atomic step: with a separate check and
/// subtraction, two calls at count 1 would wrap it to usize::MAX and the window would never hide.
fn release_modal(count: &std::sync::atomic::AtomicUsize) {
    use std::sync::atomic::Ordering::SeqCst;
    let _ = count.fetch_update(SeqCst, SeqCst, |n| n.checked_sub(1));
}

/// Handle focus change event
fn handle_focus_change(window: &tauri::WebviewWindow, focused: bool) {
    // If focused, do nothing
    if focused {
        return;
    }
    let Some(state) = window.try_state::<WindowState>() else {
        return;
    };

    // If shown within 200ms, do nothing
    if let Ok(last_shown) = state.last_window_shown.lock() {
        if let Some(ref instant) = *last_shown {
            if instant.elapsed().as_millis() <= 200 {
                return;
            }
        }
    }

    // If any modal is open, do nothing
    if state.prevent_hide.load(std::sync::atomic::Ordering::SeqCst) > 0 {
        return;
    }

    // Hide window
    if let Ok(mut hidden) = state.last_window_hidden.lock() {
        *hidden = Some(std::time::Instant::now());
    }
    let _ = window.hide();
}

/// Setup window events
pub fn setup_window_events(app: &tauri::App) {
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
                    handle_focus_change(&window_clone, *focused);
                }
                _ => {}
            }
        });
        // Hide window on startup
        let _ = window.hide();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

    #[test]
    fn releasing_a_modal_never_goes_below_zero() {
        let count = AtomicUsize::new(1);
        release_modal(&count);
        release_modal(&count);
        release_modal(&count);
        assert_eq!(count.load(SeqCst), 0);
    }

    #[test]
    fn concurrent_releases_do_not_wrap_the_count() {
        let count = std::sync::Arc::new(AtomicUsize::new(1));
        let threads: Vec<_> = (0..8)
            .map(|_| {
                let count = count.clone();
                std::thread::spawn(move || release_modal(&count))
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        assert_eq!(count.load(SeqCst), 0);
    }
}
