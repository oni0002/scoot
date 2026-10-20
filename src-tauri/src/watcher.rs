use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use tauri::Emitter;

/// File watcher struct
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
}

/// File watcher
/// Watches files in one folder and asks for a reload when one of them changes. Bursts of events are folded into one
/// reload by the `settings-file-changed` listener (see `lifecycle::setup_reload_listeners`).
impl FileWatcher {
    /// `file_paths` are all in the same folder
    pub fn new<P: AsRef<Path>>(
        file_paths: &[P],
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

        let file_paths: Vec<_> = file_paths
            .iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect();
        let watch_path = file_paths
            .first()
            .and_then(|p| p.parent())
            .ok_or_else(|| crate::error::AppError::System("No file to watch".to_string()))?;

        watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| crate::error::AppError::System(format!("Failed to watch file: {}", e)))?;

        let file_names: Vec<String> = file_paths
            .iter()
            .filter_map(|p| p.file_name()?.to_str().map(str::to_string))
            .collect();

        // Process file change events in a separate thread
        thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                // Check if the event is for one of the watched files
                let is_target_file = event.paths.iter().any(|path| {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|name| file_names.iter().any(|f| f == name))
                });

                if is_target_file {
                    // Modify, Create, Rename, Remove, ... all mean "read it again". If the file
                    // was removed, ConfigStore will regenerate the defaults.
                    log::debug!("Config file event ({:?}): {:?}", event.kind, event.paths);

                    if let Err(e) = app_handle.emit(crate::lifecycle::SETTINGS_FILE_CHANGED, ()) {
                        log::error!("Failed to report a settings.json change: {}", e);
                    }
                }
            }
        });

        Ok(Self { _watcher: watcher })
    }
}
