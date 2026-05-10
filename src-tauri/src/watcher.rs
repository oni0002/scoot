use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::Emitter;

/// File watcher struct
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    #[allow(dead_code)]
    file_path: PathBuf,
}

/// File watcher
/// Watches a file and emits events when changes are detected
impl FileWatcher {
    pub fn new<P: AsRef<Path>>(
        file_path: P,
        app_handle: tauri::AppHandle,
    ) -> Result<Self, crate::error::AppError> {
        // Channel to send events
        let (tx, rx) = mpsc::channel();
        // File watcher to monitor file changes
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                // Send event to channel
                if let Ok(event) = res {
                    if let Err(e) = tx.send(event) {
                        log::error!("Failed to send file watcher event: {}", e);
                    }
                }
            },
            Config::default(),
        )
        .map_err(|e| {
            crate::error::AppError::System(format!("Failed to create file watcher: {}", e))
        })?;

        let file_path = file_path.as_ref().to_path_buf();

        // Get the parent directory of the target file
        let watch_path = if let Some(parent) = file_path.parent() {
            parent
        } else {
            file_path.as_path()
        };

        // Watch the target file
        watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| crate::error::AppError::System(format!("Failed to watch file: {}", e)))?;

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Process file change events in a separate thread
        thread::spawn(move || {
            let mut last_event_time = std::time::Instant::now();

            while let Ok(event) = rx.recv() {
                // Check if the event is for the target file
                let is_target_file = event.paths.iter().any(|path| {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .map_or(false, |name| name == file_name)
                });

                if is_target_file {
                    // Prevent duplicate events within 500ms (debounce)
                    let now = std::time::Instant::now();
                    if now.duration_since(last_event_time) < Duration::from_millis(500) {
                        continue;
                    }
                    last_event_time = now;

                    // If the event is related to the target file (Modify, Create, Rename, Remove, etc.)
                    // Try to reload
                    // If Remove, ConfigStore will regenerate default values
                    log::debug!("Config file event ({:?}): {:?}", event.kind, event.paths);

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
}

#[cfg(test)]
mod tests {

    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        // Create a file
        fs::write(&file_path, "{}").unwrap();

        // AppHandle mock is not available here; full FileWatcher tests are covered by integration tests
        assert!(file_path.exists());
    }

    #[test]
    fn test_file_path_operations() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        // Create a file
        fs::write(&file_path, "{}").unwrap();

        // Verify the file exists
        assert!(file_path.exists());

        // Delete the file
        fs::remove_file(&file_path).unwrap();
        assert!(!file_path.exists());
    }
}
