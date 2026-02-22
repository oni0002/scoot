use crate::store::state::AppState;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// グローバルショートカットを登録
///
/// ## args
///
/// * `app_handle` - TauriのAppHandle
/// * `hotkey` - 登録するホットキー
/// * `handler` - 押下時に実行するハンドラ
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

    // 既存の登録を確認
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

    // 既存のショートカットを解除
    unregister(app_handle);

    // イベントハンドラの登録
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

    // OSにショートカットを登録
    match app_handle.global_shortcut().register(hotkey) {
        Ok(_) => {
            // Stateを更新
            if let Some(state) = app_handle.try_state::<AppState>() {
                if let Ok(mut registered) = state.shortcut.lock() {
                    *registered = Some(hotkey.to_string());
                }
            }
            let _ = app_handle.emit("shortcut-registered", hotkey);
            log::info!("Global shortcut registered: {}", hotkey);
            Ok(())
        }
        Err(e) => {
            // 既に登録されている場合は、成功とみなす
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

/// 登録済みのショートカットを解除
pub fn unregister(app_handle: &AppHandle) {
    if let Some(state) = app_handle.try_state::<AppState>() {
        if let Ok(mut reg) = state.shortcut.lock() {
            if let Some(hotkey) = reg.as_ref() {
                log::info!("Unregistering shortcut: {}", hotkey);
                let _ = app_handle.global_shortcut().unregister(hotkey.as_str());
            }
            *reg = None;
        }
    } else {
        // State取得失敗時のフォールバック
        let _ = app_handle.global_shortcut().unregister_all();
    }
}
