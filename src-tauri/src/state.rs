use crate::commands::registry::CommandRegistry;
use crate::commands::store::CommandStore;
use crate::config::domain::Config;
use crate::config::store::ConfigStore;
use crate::watcher::FileWatcher;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct CommandsState {
    pub commands: Mutex<CommandRegistry>,
    pub command_store: CommandStore,
}

pub struct ConfigState {
    pub config: Mutex<Config>,
    pub config_store: ConfigStore,
    pub _file_watcher_keep_alive: Option<FileWatcher>,
}

pub struct ShortcutState {
    pub shortcut: Mutex<Option<String>>,
}

pub struct WindowState {
    pub last_window_shown: Arc<Mutex<Option<Instant>>>,
    pub last_window_hidden: Arc<Mutex<Option<Instant>>>,
    pub last_tray_click: Mutex<Option<Instant>>,
    pub prevent_hide: Arc<std::sync::atomic::AtomicUsize>,
}

impl CommandsState {
    pub fn new(registry: CommandRegistry, store: CommandStore) -> Self {
        Self {
            commands: Mutex::new(registry),
            command_store: store,
        }
    }
}

impl ConfigState {
    pub fn new(config: Config, store: ConfigStore, watcher: Option<FileWatcher>) -> Self {
        Self {
            config: Mutex::new(config),
            config_store: store,
            _file_watcher_keep_alive: watcher,
        }
    }
}

impl ShortcutState {
    pub fn new() -> Self {
        Self {
            shortcut: Mutex::new(None),
        }
    }
}

impl WindowState {
    pub fn new() -> Self {
        Self {
            last_window_shown: Arc::new(Mutex::new(None)),
            last_window_hidden: Arc::new(Mutex::new(None)),
            last_tray_click: Mutex::new(None),
            prevent_hide: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
}
