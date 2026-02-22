use crate::store::state::AppState;
use tauri::State;

/// 設定、コマンドをリロード
#[tauri::command]
pub async fn reload_config(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::system::reload(&app_handle).await
}

/// ファイルウォッチャーの状態を取得
#[tauri::command]
pub async fn get_file_watcher_status(
    state: State<'_, AppState>,
) -> Result<bool, crate::domain::error::AppError> {
    Ok(crate::services::system::get_file_watcher_status(&state))
}

/// READMEを開く
#[tauri::command]
pub async fn open_readme(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::system::open_readme(&app_handle)
}

/// アプリを終了
#[tauri::command]
pub async fn quit_app(app_handle: tauri::AppHandle) -> Result<(), crate::domain::error::AppError> {
    crate::services::system::quit_app(&app_handle);
    Ok(())
}
