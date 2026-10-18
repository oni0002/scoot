use crate::commands::registry::CommandRegistry;
use crate::commands::store::CommandStore;
use crate::commands::usage::UsageTracker;
use crate::config::domain::Config;
use crate::config::store::ConfigStore;
use crate::watcher::FileWatcher;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use std::time::Instant;

pub struct CommandsState {
    pub commands: Mutex<CommandRegistry>,
    pub command_store: CommandStore,
    pub usage: UsageTracker,
}

pub struct ConfigState {
    pub config: Mutex<Config>,
    pub config_store: ConfigStore,
    pub _file_watcher_keep_alive: Option<FileWatcher>,
}

#[derive(Default)]
pub struct ShortcutState {
    pub shortcut: Mutex<Option<String>>,
}

#[derive(Default)]
pub struct WindowState {
    pub last_window_shown: Mutex<Option<Instant>>,
    pub last_window_hidden: Mutex<Option<Instant>>,
    pub last_tray_click: Mutex<Option<Instant>>,
    /// How many dialogs or screens hold the window open; it hides itself only at zero
    pub prevent_hide: AtomicUsize,
}

impl CommandsState {
    pub fn new(registry: CommandRegistry, store: CommandStore) -> Self {
        Self {
            commands: Mutex::new(registry),
            command_store: store,
            usage: UsageTracker::new(),
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
