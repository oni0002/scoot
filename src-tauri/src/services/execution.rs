use crate::domain::command::Command;

/// コマンドの実行
pub async fn execute_command(
    app_handle: &tauri::AppHandle,
    command: &Command,
    args: &[String],
) -> Result<String, crate::domain::error::AppError> {
    // コマンドを構築 (引数があれば展開)
    let final_command = if command.has_placeholders() {
        command.substitute_args(args)
    } else {
        command.command.clone()
    };

    // Scootコマンドの処理
    if command.category == crate::domain::command::CATEGORY_SCOOT
        || final_command.starts_with("scoot://")
    {
        return execute_scoot_command(app_handle, &final_command).await;
    }

    // 他のコマンド
    match command.category.as_str() {
        // URL, ブックマーク
        crate::domain::command::CATEGORY_URL | crate::domain::command::CATEGORY_BOOKMARK => {
            execute_url(app_handle, &final_command).await
        }
        // ファイル, アプリケーション
        crate::domain::command::CATEGORY_FILE | crate::domain::command::CATEGORY_APPLICATION => {
            execute_local_file(app_handle, &final_command).await
        }
        // シェルコマンド
        crate::domain::command::CATEGORY_COMMAND => {
            crate::infra::system::execute_shell_command(
                &final_command,
                &command.working_dir,
                command.show_window.unwrap_or(false),
            )
            .await
        }
        // その他のカテゴリはシェルコマンドとして扱うデフォルト挙動
        _ => {
            crate::infra::system::execute_shell_command(
                &final_command,
                &command.working_dir,
                command.show_window.unwrap_or(false),
            )
            .await
        }
    }
}

/// scootの内部コマンドを実行
async fn execute_scoot_command(
    app_handle: &tauri::AppHandle,
    command: &str,
) -> Result<String, crate::domain::error::AppError> {
    log::debug!("Executing scoot command: {}", command);

    match command {
        "scoot://add-command" => {
            crate::services::system::open_add_command_dialog(app_handle)?;
            Ok("Opening add command dialog".to_string())
        }
        "scoot://open-commands" => {
            crate::services::system::open_commands_json(app_handle)?;
            Ok("Opened commands.json".to_string())
        }
        "scoot://open-config" => {
            crate::services::system::open_config_json(app_handle)?;
            Ok("Opened config.json".to_string())
        }
        "scoot://open-readme" => {
            crate::services::system::open_readme(app_handle)?;
            Ok("Opened README.md".to_string())
        }
        "scoot://open-log" => {
            crate::services::system::open_log_directory(app_handle)?;
            Ok("Log directory opened".to_string())
        }
        "scoot://reload" => crate::services::system::reload(app_handle)
            .await
            .map(|_| "Configuration and commands reloaded".to_string()),
        "scoot://kill" => {
            crate::services::system::quit_app(app_handle);
            Ok("Application terminated".to_string())
        }
        _ => Err(crate::domain::error::AppError::CommandExecution(format!(
            "Unknown scoot command: {}",
            command
        ))),
    }
}

/// URLカテゴリのコマンドを実行
async fn execute_url(
    app_handle: &tauri::AppHandle,
    url: &str,
) -> Result<String, crate::domain::error::AppError> {
    log::debug!("Opening URL: {}", url);

    crate::infra::system::open_url(app_handle, url).map_err(|e| {
        let error_msg = format!(
            "Failed to open URL '{}': {}. Please check if a web browser or associated application is installed.",
            url, e
        );
        log::error!("Error: {}", error_msg);
        crate::domain::error::AppError::CommandExecution(error_msg)
    })?;

    let success_msg = format!("Successfully opened URL: {}", url);
    log::debug!("{}", success_msg);
    Ok(success_msg)
}

/// ファイルパスを開く
async fn execute_local_file(
    app_handle: &tauri::AppHandle,
    file_path: &str,
) -> Result<String, crate::domain::error::AppError> {
    log::debug!("Opening file: {}", file_path);

    // 環境変数の展開
    let expanded_path = crate::infra::env::expand_env_vars(file_path);
    log::debug!("Opening file (expanded): {}", expanded_path);

    // ファイルの存在確認
    if !std::path::Path::new(&expanded_path).exists() {
        let error_msg = format!(
            "File not found: '{}' (expanded from '{}'). Please check if the file exists and the path is correct.",
            expanded_path, file_path
        );
        log::error!("Error: {}", error_msg);
        return Err(crate::domain::error::AppError::CommandExecution(error_msg));
    }

    // ファイルを開く
    crate::infra::system::open_path(app_handle, &expanded_path).map_err(|e| {
        let error_msg = format!("Failed to open file '{}': {}. Please check file permissions and ensure a default application is set for this file type.", expanded_path, e);
        log::error!("Error: {}", error_msg);
        crate::domain::error::AppError::CommandExecution(error_msg)
    })?;

    let success_msg = format!("Successfully opened file: {}", expanded_path);
    log::debug!("{}", success_msg);
    Ok(success_msg)
}
