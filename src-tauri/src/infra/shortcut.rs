use crate::store::state::AppState;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Register a global shortcut
///
/// ## args
///
/// * `app_handle` - Tauri AppHandle
/// * `hotkey` - The hotkey to register
/// * `handler` - The handler to execute when the hotkey is pressed
///
/// ## returns
///
/// * `Result<(), Box<dyn std::error::Error>>`
pub fn register<F>(
    app_handle: &AppHandle,
    hotkey: &str,
    handler: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(&AppHandle) + Send + Sync + 'static,
{
    let app_handle_clone = app_handle.clone();

    // Check if the shortcut is already registered
    if let Some(state) = app_handle.try_state::<AppState>() {
        let lock = match state.shortcut.lock() {
            Ok(l) => l,
            Err(e) => {
                log::error!("Failed to lock shortcut state: {}", e);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )));
            }
        };
        if let Some(current) = lock.as_ref() {
            if current == hotkey && app_handle.global_shortcut().is_registered(hotkey) {
                log::debug!("Shortcut {} is already registered.", hotkey);
                return Ok(());
            }
        }
    }

    // Unregister existing shortcut
    unregister(app_handle);

    // Register event handler
    if let Err(e) =
        app_handle
            .global_shortcut()
            .on_shortcut(hotkey, move |_app, _shortcut, event| {
                if matches!(event.state, ShortcutState::Pressed) {
                    handler(&app_handle_clone);
                }
            })
    {
        log::error!("Failed to set shortcut handler for {}: {}", hotkey, e);
        return Err(Box::new(e));
    }

    // Register the shortcut with the OS
    match app_handle.global_shortcut().register(hotkey) {
        Ok(_) => {
            // Update the state
            if let Some(state) = app_handle.try_state::<AppState>() {
                if let Ok(mut registered) = state.shortcut.lock() {
                    *registered = Some(hotkey.to_string());
                }
            }
            let _ = app_handle.emit("shortcut-registered", hotkey);
            log::debug!("Global shortcut registered: {}", hotkey);
            Ok(())
        }
        Err(e) => {
            // If the hotkey is already registered, consider it a success
            if app_handle.global_shortcut().is_registered(hotkey) {
                if let Some(state) = app_handle.try_state::<AppState>() {
                    if let Ok(mut registered) = state.shortcut.lock() {
                        *registered = Some(hotkey.to_string());
                    }
                }
                Ok(())
            } else {
                log::error!("Failed to register hotkey {}: {}", hotkey, e);
                let _ = app_handle.emit("shortcut-warning", format!("{} not available", hotkey));
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )))
            }
        }
    }
}

/// Unregister the shortcut
pub fn unregister(app_handle: &AppHandle) {
    if let Some(state) = app_handle.try_state::<AppState>() {
        if let Ok(mut reg) = state.shortcut.lock() {
            if let Some(hotkey) = reg.as_ref() {
                log::debug!("Unregistering shortcut: {}", hotkey);
                let _ = app_handle.global_shortcut().unregister(hotkey.as_str());
            }
            *reg = None;
        }
    } else {
        // Fallback if state cannot be retrieved
        let _ = app_handle.global_shortcut().unregister_all();
    }
}
