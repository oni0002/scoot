use crate::domain::command::{Command, Commands};
use crate::store::state::AppState;
use tauri::State;

/// すべてのコマンドを取得
#[tauri::command]
pub async fn get_all_commands(
    state: State<'_, AppState>,
) -> Result<Vec<Command>, crate::domain::error::AppError> {
    Ok(crate::services::command::get_all(&state.commands))
}

/// コマンドを追加
#[tauri::command]
pub async fn add_command(
    command: Command,
    state: State<'_, AppState>,
) -> Result<String, crate::domain::error::AppError> {
    crate::services::command::add(&state.config_manager, &state.commands, command).await
}

/// コマンドを更新
#[tauri::command]
pub async fn update_command(
    command: Command,
    state: State<'_, AppState>,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::command::update(&state.config_manager, &state.commands, command).await
}

/// コマンドを削除
#[tauri::command]
pub async fn delete_command(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::command::delete(&state.config_manager, &state.commands, &id).await
}

/// コマンドを実行(非同期)
#[tauri::command]
pub async fn execute_command(
    command: Command,
    args: Vec<String>,
    app_handle: tauri::AppHandle,
    _state: State<'_, AppState>,
) -> Result<String, crate::domain::error::AppError> {
    crate::services::execution::execute_command(&app_handle, &command, &args).await
}

/// プロンプトでコマンドを検索
#[tauri::command]
pub async fn get_commands_by_prompt(
    prompt: String,
    state: State<'_, AppState>,
) -> Result<Vec<Command>, crate::domain::error::AppError> {
    Ok(crate::services::command::get_by_prompt(
        &state.commands,
        &prompt,
    ))
}

/// コマンドを取得
#[tauri::command]
pub async fn get_commands(
    state: State<'_, AppState>,
) -> Result<Commands, crate::domain::error::AppError> {
    crate::services::command::get_commands(&state.config_manager).await
}

/// コマンドを保存
#[tauri::command]
pub async fn save_commands(
    commands: Commands,
    state: State<'_, AppState>,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::command::save_commands(&state.config_manager, &state.commands, &commands).await
}

/// commands.jsonのパスを取得
#[tauri::command]
pub async fn get_commands_file_path(
    state: State<'_, AppState>,
) -> Result<String, crate::domain::error::AppError> {
    Ok(crate::services::command::get_file_path(
        &state.config_manager,
    ))
}

/// commands.jsonのスキーマを取得
#[tauri::command]
pub async fn get_commands_schema() -> Result<serde_json::Value, crate::domain::error::AppError> {
    Ok(crate::domain::config::generate_commands_schema())
}

/// commands.jsonを検証
#[tauri::command]
pub async fn validate_commands(
    config: serde_json::Value,
) -> Result<serde_json::Value, crate::domain::error::AppError> {
    match crate::domain::config::commands_from_json_with_validation(&config.to_string()) {
        Ok(_) => Ok(serde_json::json!({ "valid": true, "errors": [] })),
        Err(error) => Ok(serde_json::json!({ "valid": false, "errors": [error] })),
    }
}

/// commands.jsonを開く
#[tauri::command]
pub async fn open_commands_json(
    app_handle: tauri::AppHandle,
) -> Result<(), crate::domain::error::AppError> {
    crate::services::system::open_commands_json(&app_handle)
}
