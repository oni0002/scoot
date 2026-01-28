use crate::domain::command::{Command, Commands};
use crate::store::state::AppState;
use tauri::State;

/// すべてのコマンドを取得
#[tauri::command]
pub async fn get_all_commands(state: State<'_, AppState>) -> Result<Vec<Command>, String> {
    Ok(crate::services::command::get_all(&state))
}

/// コマンドを追加
#[tauri::command]
pub async fn add_command(command: Command, state: State<'_, AppState>) -> Result<String, String> {
    crate::services::command::add(&state, command).await
}

/// コマンドを更新
#[tauri::command]
pub async fn update_command(command: Command, state: State<'_, AppState>) -> Result<(), String> {
    crate::services::command::update(&state, command).await
}

/// コマンドを削除
#[tauri::command]
pub async fn delete_command(id: String, state: State<'_, AppState>) -> Result<(), String> {
    crate::services::command::delete(&state, &id).await
}

/// コマンドを実行(非同期)
#[tauri::command]
pub async fn execute_command(
    command: Command,
    args: Vec<String>,
    app_handle: tauri::AppHandle,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    crate::services::execution::execute_command(&app_handle, &command, &args).await
}

/// プロンプトでコマンドを検索
#[tauri::command]
pub async fn get_commands_by_prompt(
    prompt: String,
    state: State<'_, AppState>,
) -> Result<Vec<Command>, String> {
    Ok(crate::services::command::get_by_prompt(&state, &prompt))
}

/// コマンドを取得
#[tauri::command]
pub async fn get_commands(state: State<'_, AppState>) -> Result<Commands, String> {
    crate::services::command::get_commands(&state).await
}

/// コマンドを保存
#[tauri::command]
pub async fn save_commands(commands: Commands, state: State<'_, AppState>) -> Result<(), String> {
    crate::services::command::save_commands(&state, &commands).await
}

/// commands.jsonのパスを取得
#[tauri::command]
pub async fn get_commands_file_path(state: State<'_, AppState>) -> Result<String, String> {
    Ok(crate::services::command::get_file_path(&state))
}

/// commands.jsonのスキーマを取得
#[tauri::command]
pub async fn get_commands_schema() -> Result<serde_json::Value, String> {
    Ok(crate::domain::config::generate_commands_schema())
}

/// commands.jsonを検証
#[tauri::command]
pub async fn validate_commands(config: serde_json::Value) -> Result<serde_json::Value, String> {
    match crate::domain::config::commands_from_json_with_validation(&config.to_string()) {
        Ok(_) => Ok(serde_json::json!({ "valid": true, "errors": [] })),
        Err(error) => Ok(serde_json::json!({ "valid": false, "errors": [error] })),
    }
}

/// commands.jsonを開く
#[tauri::command]
pub async fn open_commands_json(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::services::system::open_commands_json(&app_handle)
}
