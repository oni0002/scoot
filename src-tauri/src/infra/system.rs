use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

/// 指定されたパス(ファイル、ディレクトリ、URL)をデフォルトのアプリケーションで開く
pub fn open_path(app_handle: &AppHandle, path: &str) -> Result<(), String> {
    app_handle
        .opener()
        .open_path(path, None::<&str>)
        .map_err(|e| format!("Failed to open path '{}': {}", path, e))
}

/// 指定されたURLをデフォルトのブラウザで開く
pub fn open_url(app_handle: &AppHandle, url: &str) -> Result<(), String> {
    app_handle
        .opener()
        .open_url(url, None::<&str>)
        .map_err(|e| format!("Failed to open URL '{}': {}", url, e))
}

/// ファイルが存在することを確認し、なければデフォルトの内容で作成する
pub fn ensure_file_exists<F>(path: &Path, create_content: F) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    if !path.exists() {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
        create_content()?;
    }
    Ok(())
}

/// ディレクトリが存在することを確認し、なければ作成する
pub fn ensure_directory_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// リソースファイルのパスを解決する
pub fn resolve_resource(app_handle: &AppHandle, path: &str) -> Result<PathBuf, String> {
    let resource_path = app_handle
        .path()
        .resolve(path, tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve resource path '{}': {}", path, e))?;

    if !resource_path.exists() {
        return Err(format!("Resource not found: {}", path));
    }

    Ok(resource_path)
}

/// ログディレクトリのパスを取得する
pub fn get_log_dir(_app_handle: &AppHandle) -> Result<PathBuf, String> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .ok_or_else(|| "Failed to determine log directory".to_string())
}

/// シェルコマンドを実行
pub async fn execute_shell_command(
    command: &str,
    working_dir: &Option<String>,
) -> Result<String, String> {
    use std::process::Command as StdCommand;
    log::info!("Executing shell command: {}", command);

    // コマンドが空の場合はエラー
    if command.trim().is_empty() {
        return Err("System command cannot be empty.".to_string());
    }

    // コマンドを構築
    let mut cmd_builder = if cfg!(target_os = "windows") {
        let mut cmd = StdCommand::new("cmd");
        cmd.args([
            "/C",
            "start",           // 新しいウィンドウを開く
            "Scoot Execution", // タイトル
            "powershell",      // PowerShellを使用
            "-Command",        // コマンドを実行
            command,           // 実行するコマンド
        ]);
        cmd
    } else {
        let mut cmd = StdCommand::new("sh");
        cmd.args(["-c", command]);
        cmd
    };

    // ワークディレクトリを設定
    if let Some(dir) = working_dir {
        if !dir.trim().is_empty() {
            if std::path::Path::new(dir).exists() {
                cmd_builder.current_dir(dir);
            } else {
                // ワークディレクトリが存在しない場合は警告
                log::warn!(
                    "Warning: Working directory '{}' does not exist. Ignoring.",
                    dir
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
            Err(error_msg)
        }
    }
}
