use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

/// 指定されたパス(ファイル、ディレクトリ、URL)をデフォルトのアプリケーションで開く
pub fn open_path(app_handle: &AppHandle, path: &str) -> Result<(), crate::domain::error::AppError> {
    app_handle
        .opener()
        .open_path(path, None::<&str>)
        .map_err(|e| {
            crate::domain::error::AppError::System(format!("Failed to open path '{}': {}", path, e))
        })
}

/// 指定されたURLをデフォルトのブラウザで開く
pub fn open_url(app_handle: &AppHandle, url: &str) -> Result<(), crate::domain::error::AppError> {
    app_handle
        .opener()
        .open_url(url, None::<&str>)
        .map_err(|e| {
            crate::domain::error::AppError::System(format!("Failed to open URL '{}': {}", url, e))
        })
}

/// ファイルが存在することを確認し、なければデフォルトの内容で作成する
pub fn ensure_file_exists<F>(
    path: &Path,
    create_content: F,
) -> Result<(), crate::domain::error::AppError>
where
    F: FnOnce() -> Result<(), crate::domain::error::AppError>,
{
    if !path.exists() {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
            }
        }
        create_content()?;
    }
    Ok(())
}

/// ディレクトリが存在することを確認し、なければ作成する
pub fn ensure_directory_exists(path: &Path) -> Result<(), crate::domain::error::AppError> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
    }
    Ok(())
}

/// リソースファイルのパスを解決する
pub fn resolve_resource(
    app_handle: &AppHandle,
    path: &str,
) -> Result<PathBuf, crate::domain::error::AppError> {
    let resource_path = app_handle
        .path()
        .resolve(path, tauri::path::BaseDirectory::Resource)
        .map_err(|e| {
            crate::domain::error::AppError::System(format!(
                "Failed to resolve resource path '{}': {}",
                path, e
            ))
        })?;

    if !resource_path.exists() {
        return Err(crate::domain::error::AppError::NotFound(format!(
            "Resource not found: {}",
            path
        )));
    }

    Ok(resource_path)
}

/// ログディレクトリのパスを取得する
pub fn get_log_dir(_app_handle: &AppHandle) -> Result<PathBuf, crate::domain::error::AppError> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .ok_or_else(|| {
            crate::domain::error::AppError::System("Failed to determine log directory".to_string())
        })
}

/// シェルコマンドを実行
pub async fn execute_shell_command(
    command: &str,
    working_dir: &Option<String>,
    show_window: bool,
) -> Result<String, crate::domain::error::AppError> {
    use std::process::Command as StdCommand;
    log::info!("Executing shell command: {}", command);

    // コマンドが空の場合はエラー
    if command.trim().is_empty() {
        return Err(crate::domain::error::AppError::Validation(
            "System command cannot be empty.".to_string(),
        ));
    }

    // コマンドを構築
    let mut cmd_builder = if cfg!(target_os = "windows") {
        use std::os::windows::process::CommandExt;
        // PowerShellを直接呼び出す
        let mut cmd = StdCommand::new("powershell");
        // プロファイル読み込みをスキップして高速化
        let mut args = vec!["-NoProfile"];
        // show_windowがtrueの場合 -NoExit を追加 (ウィンドウを閉じないようにする)
        if show_window {
            args.push("-NoExit");
        }

        args.push("-Command");
        args.push(command);

        cmd.args(args);

        if show_window {
            // 新しいコンソールウィンドウを作成 (CREATE_NEW_CONSOLE)
            cmd.creation_flags(0x00000010);
        } else {
            // ウィンドウを表示しない (CREATE_NO_WINDOW)
            cmd.creation_flags(0x08000000);
        }
        cmd
    } else {
        let mut cmd = StdCommand::new("sh");
        cmd.args(["-c", command]);
        cmd
    };

    // ワークディレクトリを設定
    if let Some(dir) = working_dir {
        // ダブルクォートで囲まれている場合は除去する
        let trimmed_dir = dir.trim();
        let clean_dir =
            if trimmed_dir.starts_with('"') && trimmed_dir.ends_with('"') && trimmed_dir.len() >= 2
            {
                &trimmed_dir[1..trimmed_dir.len() - 1]
            } else {
                trimmed_dir
            };

        if !clean_dir.is_empty() {
            if std::path::Path::new(clean_dir).exists() {
                cmd_builder.current_dir(clean_dir);
            } else {
                // ワークディレクトリが存在しない場合は警告
                log::warn!(
                    "Warning: Working directory '{}' does not exist. Ignoring.",
                    clean_dir
                );
            }
        }
    }

    // 非同期でコマンドを実行
    match cmd_builder.spawn() {
        Ok(_) => {
            let success_msg = "Command launched successfully (background).".to_string();
            log::info!("{}", success_msg);
            Ok(success_msg)
        }
        Err(e) => {
            let error_msg = format!("Failed to spawn system command '{}': {}.", command, e);
            log::error!("Error: {}", error_msg);
            Err(crate::domain::error::AppError::CommandExecution(error_msg))
        }
    }
}
