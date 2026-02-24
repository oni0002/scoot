use crate::store::state::AppState;
use tauri::State;

/// Toggle window visibility
#[tauri::command]
pub async fn toggle_window(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::window::toggle_visibility(&app_handle)
}

/// Hide window
#[tauri::command]
pub async fn hide_window(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::window::hide(&app_handle)
}

/// Show window
#[tauri::command]
pub async fn show_window(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::window::show(&app_handle)
}

/// Set prevent_hide flag
#[tauri::command]
pub async fn set_prevent_hide(
    prevent: bool,
    state: State<'_, AppState>,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::window::set_prevent_hide(&state.prevent_hide, prevent)
}
