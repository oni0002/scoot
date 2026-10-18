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
/// Watches a file and asks for a reload when it changes. Bursts of events are folded into one
/// reload by the `request-reload` listener (see `lifecycle::setup_reload_listeners`).
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
            while let Ok(event) = rx.recv() {
                // Check if the event is for the target file
                let is_target_file = event.paths.iter().any(|path| {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|name| name == file_name)
                });

                if is_target_file {
                    // Modify, Create, Rename, Remove, ... all mean "read it again". If the file
                    // was removed, ConfigStore will regenerate the defaults.
                    log::debug!("Config file event ({:?}): {:?}", event.kind, event.paths);

                    if let Err(e) = app_handle.emit("request-reload", ()) {
                        log::error!("Failed to request a reload: {}", e);
                    }
                }
            }
        });

        Ok(Self { _watcher: watcher })
    }
}
