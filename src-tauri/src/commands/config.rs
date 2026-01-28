use crate::domain::config::Config;
use crate::store::state::AppState;
use tauri::State;

/// 設定を取得
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Config, String> {
    crate::services::config::get(&state)
}

/// 設定を保存
#[tauri::command]
pub async fn save_config(config: Config, state: State<'_, AppState>) -> Result<(), String> {
    crate::services::config::save(&state, &config).await
}

/// config.jsonのパスを取得
#[tauri::command]
pub async fn get_config_file_path(state: State<'_, AppState>) -> Result<String, String> {
    Ok(crate::services::config::get_file_path(&state))
}

/// config.jsonのスキーマを取得
#[tauri::command]
pub async fn get_config_schema() -> Result<serde_json::Value, String> {
    Ok(Config::generate_schema())
}

/// config.jsonを検証
#[tauri::command]
pub async fn validate_config(config: serde_json::Value) -> Result<serde_json::Value, String> {
    match Config::from_json_with_validation(&config.to_string()) {
        Ok(_) => Ok(serde_json::json!({ "valid": true, "errors": [] })),
        Err(error) => Ok(serde_json::json!({ "valid": false, "errors": [error] })),
    }
}

/// config.jsonを開く
#[tauri::command]
pub async fn open_config_json(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::services::system::open_config_json(&app_handle)
}
