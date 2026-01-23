use crate::models::Command;
use std::process::Command as StdCommand;

use tauri_plugin_opener::OpenerExt;

/// コマンドの実行
pub async fn execute_command(
    app_handle: &tauri::AppHandle,
    command: &Command,
    args: &[String],
) -> Result<String, String> {
    let final_command = if command.has_placeholders() {
        command.substitute_args(args)
    } else {
        command.command.clone()
    };

    // Scootコマンドの処理
    if command.category == crate::models::CATEGORY_SCOOT || final_command.starts_with("scoot://") {
        return execute_scoot_command(app_handle, &final_command).await;
    }

    match command.category.as_str() {
        // URL, ブックマーク
        crate::models::CATEGORY_URL | crate::models::CATEGORY_BOOKMARK => {
            execute_url(app_handle, &final_command).await
        }
        // ファイル, アプリケーション
        crate::models::CATEGORY_FILE | crate::models::CATEGORY_APPLICATION => {
            execute_local_file(app_handle, &final_command).await
        }
        // シェルコマンド
        crate::models::CATEGORY_COMMAND => {
            execute_shell_command(&final_command, &command.working_dir).await
        }
        // その他のカテゴリはシェルコマンドとして扱うデフォルト挙動
        _ => execute_shell_command(&final_command, &command.working_dir).await,
    }
}

/// scootの内部コマンドを実行
async fn execute_scoot_command(
    app_handle: &tauri::AppHandle,
    command: &str,
) -> Result<String, String> {
    log::info!("Executing scoot command: {}", command);

    match command {
        "scoot://add-command" => {
            crate::app_setup::open_add_command_dialog(app_handle)?;
            Ok("Opening add command dialog".to_string())
        }
        "scoot://open-commands" => {
            crate::app_setup::open_commands_json(app_handle)?;
            Ok("Opened commands.json".to_string())
        }
        "scoot://open-config" => {
            crate::app_setup::open_config_json(app_handle)?;
            Ok("Opened config.json".to_string())
        }
        "scoot://open-readme" => {
            crate::app_setup::open_readme(app_handle)?;
            Ok("Opened README.md".to_string())
        }
        "scoot://open-log" => {
            crate::app_setup::open_log_directory(app_handle)?;
            Ok("Log directory opened".to_string())
        }
        "scoot://reload" => crate::app_setup::reload_configuration(app_handle)
            .await
            .map(|_| "Configuration reloaded".to_string()),
        "scoot://kill" => {
            crate::app_setup::quit_application(app_handle);
            Ok("Application terminated".to_string())
        }
        _ => Err(format!("Unknown scoot command: {}", command)),
    }
}

/// URLカテゴリのコマンドを実行
async fn execute_url(app_handle: &tauri::AppHandle, url: &str) -> Result<String, String> {
    log::info!("Opening URL: {}", url);

    // URL形式の基本的な検証
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!(
            "Invalid URL format: '{}'. URLs must start with http:// or https://",
            url
        ));
    }

    app_handle
        .opener()
        .open_url(url, None::<&str>)
        .map_err(|e| {
            let error_msg = format!("Failed to open URL '{}' in default browser: {}. Please check if a web browser is installed.", url, e);
            log::error!("Error: {}", error_msg);
            error_msg
        })?;

    let success_msg = format!("Successfully opened URL: {}", url);
    log::info!("{}", success_msg);
    Ok(success_msg)
}

/// ファイルパスを開く
async fn execute_local_file(
    app_handle: &tauri::AppHandle,
    file_path: &str,
) -> Result<String, String> {
    log::info!("Opening file: {}", file_path);

    // 環境変数の展開
    let expanded_path = crate::utils::expand_env_vars(file_path);
    log::info!("Opening file (expanded): {}", expanded_path);

    // ファイルの存在確認
    if !std::path::Path::new(&expanded_path).exists() {
        let error_msg = format!(
            "File not found: '{}' (expanded from '{}'). Please check if the file exists and the path is correct.",
            expanded_path, file_path
        );
        log::error!("Error: {}", error_msg);
        return Err(error_msg);
    }

    // ファイルを開く
    app_handle
        .opener()
        .open_path(&expanded_path, None::<&str>)
        .map_err(|e| {
            let error_msg = format!("Failed to open file '{}': {}. Please check file permissions and ensure a default application is set for this file type.", expanded_path, e);
            log::error!("Error: {}", error_msg);
            error_msg
        })?;

    let success_msg = format!("Successfully opened file: {}", expanded_path);
    log::info!("{}", success_msg);
    Ok(success_msg)
}

async fn execute_shell_command(
    command: &str,
    working_dir: &Option<String>,
) -> Result<String, String> {
    log::info!("Executing shell command: {}", command);

    if command.trim().is_empty() {
        return Err("System command cannot be empty.".to_string());
    }

    let mut cmd_builder = if cfg!(target_os = "windows") {
        let mut cmd = StdCommand::new("cmd");
        cmd.args([
            "/C",
            "start",
            "Scoot Execution",
            "powershell",
            "-NoExit",
            "-Command",
            command,
        ]);
        cmd
    } else {
        let mut cmd = StdCommand::new("sh");
        cmd.args(["-c", command]);
        cmd
    };

    if let Some(dir) = working_dir {
        if !dir.trim().is_empty() {
            if std::path::Path::new(dir).exists() {
                cmd_builder.current_dir(dir);
            } else {
                log::warn!(
                    "Warning: Working directory '{}' does not exist. Ignoring.",
                    dir
                );
            }
        }
    }

    match cmd_builder.spawn() {
        Ok(_) => {
            let success_msg = "Command launched successfully (background).".to_string();
            log::info!("{}", success_msg);
            Ok(success_msg)
        }
        Err(e) => {
            let error_msg = format!("Failed to spawn system command '{}': {}.", command, e);
            log::error!("Error: {}", error_msg);
            Err(error_msg)
        }
    }
}
