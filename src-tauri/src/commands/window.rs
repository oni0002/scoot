use crate::store::state::AppState;
use tauri::State;

/// ウィンドウを表示/非表示を切り替える
#[tauri::command]
pub async fn toggle_window(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::window::toggle_visibility(&app_handle)
}

/// ウィンドウを隠す
#[tauri::command]
pub async fn hide_window(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::window::hide(&app_handle)
}

/// ウィンドウを表示
#[tauri::command]
pub async fn show_window(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::window::show(&app_handle)
}

/// prevent_hideフラグを設定
#[tauri::command]
pub async fn set_prevent_hide(
    prevent: bool,
    state: State<'_, AppState>,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::window::set_prevent_hide(&state.prevent_hide, prevent)
}
