use crate::store::state::AppState;
use tauri::{AppHandle, Manager, State};

/// ウィンドウを表示/非表示を切り替える
pub fn toggle_visibility(app_handle: &AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            crate::infra::window::hide(app_handle)
        } else {
            crate::infra::window::show(app_handle)
        }
    } else {
        Ok(())
    }
}

/// ウィンドウを隠す
pub fn hide(app_handle: &AppHandle) -> Result<(), String> {
    crate::infra::window::hide(app_handle)
}

/// ウィンドウを表示
pub fn show(app_handle: &AppHandle) -> Result<(), String> {
    crate::infra::window::show(app_handle)
}

/// prevent_hideフラグを設定
pub fn set_prevent_hide(state: &State<'_, AppState>, prevent: bool) -> Result<(), String> {
    state.set_prevent_hide(prevent)
}

/// フォーカス変更時の処理
pub fn handle_focus_change(window: &tauri::WebviewWindow, focused: bool) {
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

/// ウィンドウイベントの設定
pub fn setup_window_events(app: &tauri::App) {
    if let Some(window) = app.get_webview_window("main") {
        let window_clone = window.clone();
        window.on_window_event(move |event| {
            match event {
                // ウィンドウのクローズリクエストが送られたとき
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = hide(&window_clone.app_handle());
                }
                // フォーカスが変わったとき
                tauri::WindowEvent::Focused(focused) => {
                    handle_focus_change(&window_clone, *focused);
                }
                _ => {}
            }
        });
        // 起動時は非表示
        let _ = hide(&window.app_handle());
    }
}
