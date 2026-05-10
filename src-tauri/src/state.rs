use crate::commands::registry::CommandRegistry;
use crate::commands::store::CommandStore;
use crate::config::domain::Config;
use crate::config::store::ConfigStore;
use crate::watcher::FileWatcher;
use std::sync::Mutex;

pub struct AppState {
    pub commands: Mutex<CommandRegistry>,
    pub config: Mutex<Config>,
    pub config_store: ConfigStore,
    pub command_store: CommandStore,
    pub _config_file_watcher_keep_alive: Option<FileWatcher>,
    pub shortcut: Mutex<Option<String>>,
    pub last_window_shown: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
    pub last_window_hidden: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
    pub last_tray_click: Mutex<Option<std::time::Instant>>,
    pub prevent_hide: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl AppState {
    pub fn new(
        command_registry: CommandRegistry,
        command_store: CommandStore,
        config: Config,
        config_store: ConfigStore,
        config_file_watcher: Option<FileWatcher>,
    ) -> Self {
        Self {
            commands: Mutex::new(command_registry),
            command_store,
            config: Mutex::new(config),
            config_store,
            _config_file_watcher_keep_alive: config_file_watcher,
            shortcut: Mutex::new(None),
            last_window_shown: std::sync::Arc::new(std::sync::Mutex::new(None)),
            last_window_hidden: std::sync::Arc::new(std::sync::Mutex::new(None)),
            last_tray_click: Mutex::new(None),
            prevent_hide: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }

}
