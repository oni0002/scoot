use crate::state::AppState;
use tauri::{AppHandle, Emitter, Manager, State};

/// ウィンドウを表示してフォーカスを当てる
pub fn show_window_with_focus(app_handle: &AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("window-shown", ());

        // 表示時刻を記録
        if let Some(state) = app_handle.try_state::<AppState>() {
            if let Ok(mut last_shown) = state.last_window_shown.lock() {
                *last_shown = Some(std::time::Instant::now());
            }
        }
    }
    Ok(())
}

/// ウィンドウを隠す
pub fn hide_window_sync(app_handle: &AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}

/// ウィンドウがフォーカスを獲得したときに呼ばれる
pub fn handle_focused(window: &tauri::WebviewWindow, focused: bool) {
    // フォーカスが得られた場合は何もしない
    if focused {
        return;
    }

    if let Some(state) = window.try_state::<AppState>() {
        // 表示直後 (200ms以内) なら隠さない
        if let Ok(last_shown) = state.last_window_shown.lock() {
            if let Some(ref instant) = *last_shown {
                if instant.elapsed().as_millis() <= 200 {
                    return;
                }
            }
        }

        // prevent_hide = true なら隠さない
        if let Ok(flag) = state.prevent_hide.lock() {
            if *flag {
                return;
            }
        }
    }

    // 上記以外はウィンドウを隠す
    let _ = window.hide();
}

/// ウィンドウを表示/非表示を切り替える (ロジック)
pub fn toggle_window_visibility(app_handle: &AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            hide_window_sync(app_handle)
        } else {
            show_window_with_focus(app_handle)
        }
    } else {
        Ok(())
    }
}

// hide_window, show_window wrappers removed. Use hide_window_sync / show_window_with_focus directly.

/// prevent_hideフラグを設定 (ロジック)
pub fn set_prevent_hide_flag(prevent: bool, state: &State<'_, AppState>) -> Result<(), String> {
    if let Ok(mut flag) = state.prevent_hide.lock() {
        *flag = prevent;
    }
    Ok(())
}
