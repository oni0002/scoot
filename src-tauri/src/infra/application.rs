use crate::domain::command::Command;
use std::path::Path;
use walkdir::WalkDir;

/// 指定されたディレクトリリストからアプリケーションをスキャンする
pub async fn scan(directories: &[String], extensions: &[String]) -> Result<Vec<Command>, String> {
    let directories_clone = directories.to_vec();
    let extensions = extensions
        .iter()
        .map(|e| e.to_lowercase())
        .collect::<Vec<_>>();

    // 重いI/O処理をワーカースレッドで実行
    let commands = tokio::task::spawn_blocking(move || {
        let mut commands = Vec::new();

        for dir_path in directories_clone {
            let expanded_path = crate::infra::env::expand_env_vars(&dir_path);
            let path = Path::new(&expanded_path);

            if !path.exists() {
                continue;
            }

            // 再帰的にスキャン
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();

                // 指定された拡張子のみを対象とする
                if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    if extensions.contains(&ext_str) {
                        if let Some(command) = create_command_from_path(path) {
                            commands.push(command);
                        }
                    }
                }
            }
        }

        commands
    })
    .await
    .map_err(|e| format!("Failed to scan applications: {}", e))?;

    Ok(commands)
}

/// パスからCommandオブジェクトを生成
fn create_command_from_path(path: &Path) -> Option<Command> {
    let file_stem = path.file_stem()?.to_string_lossy().to_string();
    let full_path = path.to_string_lossy().to_string();

    // ID生成
    let id_hash = md5::compute(full_path.as_bytes());
    let id = format!("app-{:x}", id_hash);

    // 親ディレクトリ名を取得（説明用）
    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    Some(Command {
        id,
        name: file_stem,
        category: "application".to_string(), // 専用カテゴリ
        command: full_path,                  // パスそのものをコマンドとする (Windowsが解決)
        description: format!("Application in {}", parent),
        prompt: None,
        working_dir: None,
        is_editable: false,
    })
}
