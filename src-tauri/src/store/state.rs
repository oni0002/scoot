use crate::domain::config::Config;
use crate::infra::config::ConfigManager;
use crate::infra::watcher::FileWatcher;
use crate::store::commands::CommandManager;
use std::sync::Mutex;

pub struct AppState {
    pub commands: Mutex<CommandManager>,
    pub config: Mutex<Config>,
    pub config_manager: ConfigManager,
    pub _commands_file_watcher: Option<FileWatcher>,
    pub _config_file_watcher: Option<FileWatcher>,
    pub shortcut: Mutex<Option<String>>,
    pub last_window_shown: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
    pub last_window_hidden: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
    pub last_tray_click: Mutex<Option<std::time::Instant>>,
    pub prevent_hide: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl AppState {
    pub fn new(
        command_manager: CommandManager,
        config: Config,
        config_manager: ConfigManager,
        commands_file_watcher: Option<FileWatcher>,
        config_file_watcher: Option<FileWatcher>,
    ) -> Self {
        Self {
            commands: Mutex::new(command_manager),
            config: Mutex::new(config),
            config_manager,
            _commands_file_watcher: commands_file_watcher,
            _config_file_watcher: config_file_watcher,
            shortcut: Mutex::new(None),
            last_window_shown: std::sync::Arc::new(std::sync::Mutex::new(None)),
            last_window_hidden: std::sync::Arc::new(std::sync::Mutex::new(None)),
            last_tray_click: Mutex::new(None),
            prevent_hide: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }

    /// prevent_hideフラグを設定
    pub fn set_prevent_hide(&self, prevent: bool) -> Result<(), crate::domain::error::AppError> {
        if let Ok(mut flag) = self.prevent_hide.lock() {
            *flag = prevent;
        }
        Ok(())
    }
}
