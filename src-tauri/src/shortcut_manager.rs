use crate::state::AppState;
use crate::window_manager::show_window_with_focus;
use tauri::{App, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// グローバルショートカットを設定
pub fn setup_global_shortcuts(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();
    let _ = app.global_shortcut().unregister_all();

    let app_handle_for_listener = app_handle.clone();
    match app.global_shortcut().on_shortcut(
        crate::models::DEFAULT_SHORTCUT,
        move |_app, _shortcut, event| {
            if matches!(event.state, ShortcutState::Pressed) {
                if let Some(window) = app_handle_for_listener.get_webview_window("main") {
                    match window.is_visible() {
                        Ok(is_visible) => {
                            if is_visible {
                                let _ = crate::window_manager::hide_window_sync(
                                    &app_handle_for_listener,
                                );
                            } else {
                                let _ = show_window_with_focus(&app_handle_for_listener);
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        },
    ) {
        Ok(_) => match app
            .global_shortcut()
            .register(crate::models::DEFAULT_SHORTCUT)
        {
            Ok(_) => {
                if let Some(state) = app.try_state::<AppState>() {
                    if let Ok(mut registered) = state.registered_shortcut.try_lock() {
                        *registered = Some(crate::models::DEFAULT_SHORTCUT.to_string());
                    }
                }
                let _ = app_handle.emit("shortcut-registered", "Alt+Space");
            }
            Err(_) => {
                let _ = app_handle.emit("shortcut-warning", "Alt+Space not available");
            }
        },
        Err(_) => {
            let _ = app_handle.emit("shortcut-warning", "Alt+Space handler failed");
        }
    }

    Ok(())
}
