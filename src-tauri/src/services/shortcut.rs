use crate::domain::config::DEFAULT_SHORTCUT;
use crate::store::state::AppState;
use tauri::{App, Manager};

/// ショートカットのセットアップ
pub fn setup_shortcuts(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle();
    // ホットキー取得（Stateから）
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

/// グローバルショートカットを登録
pub fn setup_global_shortcuts(
    app_handle: &tauri::AppHandle,
    hotkey: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::infra::shortcut::register(app_handle, hotkey, move |handle| {
        // コールバック：ウィンドウの表示切り替え
        if let Err(e) = crate::services::window::toggle_visibility(handle) {
            log::error!("Failed to toggle window visibility via shortcut: {}", e);
        }
    })
}
