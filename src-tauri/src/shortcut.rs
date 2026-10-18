use crate::settings::domain::DEFAULT_SHORTCUT;
use tauri::{App, AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// The hotkey this app currently holds, as it was written when it was registered
fn registered_hotkey(app_handle: &AppHandle) -> Option<String> {
    let state = app_handle.try_state::<crate::state::ShortcutState>()?;
    let hotkey = state.shortcut.lock().ok()?.clone();
    hotkey
}

/// The notice for a Hotkey that could not be registered. It is kept, since the first
/// registration runs in setup, long before the window is shown.
const NOTICE_SOURCE: &str = "hotkey";

/// Note that `hotkey` is the one held now, which also means the last failure is over
fn remember(app_handle: &AppHandle, hotkey: &str) {
    if let Some(state) = app_handle.try_state::<crate::state::ShortcutState>() {
        if let Ok(mut registered) = state.shortcut.lock() {
            *registered = Some(hotkey.to_string());
        }
    }
    crate::notices::clear(app_handle, NOTICE_SOURCE);
}

/// Register a global shortcut, replacing the one registered before.
///
/// The new key is taken first and the old one released only once that worked, so a key another
/// app holds never leaves Scoot without a hotkey. A key that cannot be taken is reported to the
/// webview as `shortcut-warning`; success is silent, since this also runs when a hotkey capture
/// ends and on every reload.
///
/// ## args
///
/// * `app_handle` - Tauri AppHandle
/// * `hotkey` - The hotkey to register
/// * `handler` - The handler to execute when the hotkey is pressed
pub fn register<F>(
    app_handle: &AppHandle,
    hotkey: &str,
    handler: F,
) -> Result<(), crate::error::AppError>
where
    F: Fn(&AppHandle) + Send + Sync + 'static,
{
    let previous = registered_hotkey(app_handle);
    if previous.as_deref() == Some(hotkey) && app_handle.global_shortcut().is_registered(hotkey) {
        log::debug!("Shortcut {} is already registered.", hotkey);
        return Ok(());
    }

    let app_handle_clone = app_handle.clone();
    // on_shortcut registers the key with the OS as well as the handler
    let registration =
        app_handle
            .global_shortcut()
            .on_shortcut(hotkey, move |_app, _shortcut, event| {
                if matches!(event.state, ShortcutState::Pressed) {
                    handler(&app_handle_clone);
                }
            });

    match registration {
        Ok(()) => {
            if let Some(old) = previous.filter(|old| old != hotkey) {
                log::debug!("Unregistering shortcut: {}", old);
                let _ = app_handle.global_shortcut().unregister(old.as_str());
            }
            remember(app_handle, hotkey);
            log::debug!("Global shortcut registered: {}", hotkey);
            Ok(())
        }
        // The same key written differently ("alt+space"): already ours
        Err(_) if app_handle.global_shortcut().is_registered(hotkey) => {
            remember(app_handle, hotkey);
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to register hotkey {}: {}", hotkey, e);
            crate::notices::raise(
                app_handle,
                NOTICE_SOURCE,
                crate::notices::WARNING,
                format!("{hotkey} is not available. Pick another Hotkey in Settings."),
            );
            Err(crate::error::AppError::System(e.to_string()))
        }
    }
}

/// Let go of the hotkey while one is being captured: a registered key never reaches the window.
/// `resume` takes it again.
pub fn suspend(app_handle: &AppHandle) {
    if let Some(held) = app_handle
        .try_state::<crate::state::ShortcutState>()
        .and_then(|state| state.suspend())
    {
        log::debug!("Unregistering shortcut: {}", held);
        let _ = app_handle.global_shortcut().unregister(held.as_str());
    }
}

/// Take back the hotkey `suspend` let go of. It is the one that worked before, not the saved
/// one: the saved hotkey can be unavailable, and an edit made in the settings is not saved yet.
pub fn resume(app_handle: &AppHandle) {
    let resumed = app_handle
        .try_state::<crate::state::ShortcutState>()
        .and_then(|state| state.resume());
    if let Some(hotkey) = resumed {
        if let Err(e) = setup_global_shortcuts(app_handle, &hotkey) {
            log::error!("Failed to take the hotkey back after a capture: {}", e);
        }
    }
}

/// The hotkey from the loaded settings, or the default when it cannot be read
pub fn configured_hotkey(app_handle: &AppHandle) -> String {
    app_handle
        .try_state::<crate::state::SettingsState>()
        .and_then(|state| state.settings.lock().ok().map(|c| c.hotkey.clone()))
        .unwrap_or_else(|| DEFAULT_SHORTCUT.to_string())
}

pub fn setup_shortcuts(app: &App) -> Result<(), crate::error::AppError> {
    let handle = app.handle();
    setup_global_shortcuts(handle, &configured_hotkey(handle))
}

/// Register global shortcuts
pub fn setup_global_shortcuts(
    app_handle: &tauri::AppHandle,
    hotkey: &str,
) -> Result<(), crate::error::AppError> {
    register(app_handle, hotkey, move |h| {
        // The hotkey must not hide the window from under an open dialog
        if crate::window::is_main_window_visible(h) && crate::window::is_modal_open(h) {
            return;
        }
        crate::window::toggle_main_window(h);
    })
}
