use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::Emitter;

/// ファイルウォッチャーの構造体
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    #[allow(dead_code)]
    file_path: PathBuf,
}

/// ファイルウォッチャー
/// ファイルを監視し、変更があった場合にイベントを発行する
impl FileWatcher {
    pub fn new<P: AsRef<Path>>(
        file_path: P,
        app_handle: tauri::AppHandle,
    ) -> Result<Self, crate::domain::error::AppError> {
        // チャンネルを生成
        let (tx, rx) = mpsc::channel();
        // ファイルウォッチャーを生成
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                // イベントを受け取ったらチャンネルに送信
                if let Ok(event) = res {
                    if let Err(e) = tx.send(event) {
                        log::error!("Failed to send file watcher event: {}", e);
                    }
                }
            },
            Config::default(),
        )
        .map_err(|e| {
            crate::domain::error::AppError::System(format!("Failed to create file watcher: {}", e))
        })?;

        let file_path = file_path.as_ref().to_path_buf();

        // 対象ファイルの親ディレクトリを取得
        let watch_path = if let Some(parent) = file_path.parent() {
            parent
        } else {
            file_path.as_path()
        };

        // 対象ファイルを監視
        watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| {
                crate::domain::error::AppError::System(format!("Failed to watch file: {}", e))
            })?;

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // 別スレッドでファイル変更イベントを処理
        thread::spawn(move || {
            let mut last_event_time = std::time::Instant::now();

            while let Ok(event) = rx.recv() {
                // 監視対象ファイルの変更かチェック
                let is_target_file = event.paths.iter().any(|path| {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .map_or(false, |name| name == file_name)
                });

                if is_target_file {
                    // 500ms以内の重複イベントを防ぐ (デバウンス)
                    let now = std::time::Instant::now();
                    if now.duration_since(last_event_time) < Duration::from_millis(500) {
                        continue;
                    }
                    last_event_time = now;

                    // 対象ファイルに関連するイベント(Modify, Create, Rename, Removeなど)であればリロードを試みる
                    // Removeの場合はConfigManagerがデフォルト値を再生成する挙動になる
                    log::info!("Config file event ({:?}): {:?}", event.kind, event.paths);

                    if let Err(e) = app_handle.emit("config-file-changed", ()) {
                        log::error!("Failed to emit config file changed event: {}", e);
                    }
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            file_path: file_path.clone(),
        })
    }

    /// 監視対象ファイルのパスを取得
    #[allow(dead_code)]
    pub fn get_file_path(&self) -> &Path {
        &self.file_path
    }

    /// ファイルが存在するかチェック
    #[allow(dead_code)]
    pub fn file_exists(&self) -> bool {
        self.file_path.exists()
    }

    /// ファイルの最終更新時刻を取得
    #[allow(dead_code)]
    pub fn get_last_modified(&self) -> Result<std::time::SystemTime, std::io::Error> {
        let metadata = std::fs::metadata(&self.file_path)?;
        metadata.modified()
    }
}

#[cfg(test)]
mod tests {

    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        // ファイルを作成
        fs::write(&file_path, "{}").unwrap();

        // モックのAppHandleが必要なため、実際のファイルウォッチャーのテストは統合テストで行う
        assert!(file_path.exists());
    }

    #[test]
    fn test_file_path_operations() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        // ファイルを作成
        fs::write(&file_path, "{}").unwrap();

        // ファイルが存在することを確認
        assert!(file_path.exists());

        // ファイルを削除
        fs::remove_file(&file_path).unwrap();
        assert!(!file_path.exists());
    }
}
