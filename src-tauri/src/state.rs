use crate::commands::registry::CommandRegistry;
use crate::commands::store::CommandStore;
use crate::commands::usage::UsageTracker;
use crate::settings::domain::Settings;
use crate::settings::store::SettingsStore;
use crate::watcher::FileWatcher;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use std::time::Instant;

pub struct CommandsState {
    pub commands: Mutex<CommandRegistry>,
    pub command_store: CommandStore,
    pub usage: UsageTracker,
}

pub struct SettingsState {
    pub settings: Mutex<Settings>,
    pub settings_store: SettingsStore,
    pub _file_watcher_keep_alive: Option<FileWatcher>,
}

#[derive(Default)]
pub struct ShortcutState {
    /// The hotkey this app holds now, as it was written when it was registered
    pub shortcut: Mutex<Option<String>>,
    /// The hotkey let go of while a hotkey capture runs, to be taken again when it ends
    suspended: Mutex<Option<String>>,
}

impl ShortcutState {
    /// Let go of the held hotkey and remember it. Returns it, so the caller can unregister it.
    /// Nothing is overwritten when no hotkey is held (a capture already running).
    pub fn suspend(&self) -> Option<String> {
        let held = self.shortcut.lock().ok()?.take()?;
        if let Ok(mut suspended) = self.suspended.lock() {
            *suspended = Some(held.clone());
        }
        Some(held)
    }

    /// The hotkey to take again after a capture. None when nothing was let go of, or when a
    /// hotkey has been registered in the meantime (a reload): that one is the current one.
    pub fn resume(&self) -> Option<String> {
        let suspended = self.suspended.lock().ok()?.take()?;
        let nothing_held = self.shortcut.lock().ok()?.is_none();
        nothing_held.then_some(suspended)
    }
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

impl SettingsState {
    pub fn new(settings: Settings, store: SettingsStore, watcher: Option<FileWatcher>) -> Self {
        Self {
            settings: Mutex::new(settings),
            settings_store: store,
            _file_watcher_keep_alive: watcher,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holding(hotkey: &str) -> ShortcutState {
        let state = ShortcutState::default();
        *state.shortcut.lock().unwrap() = Some(hotkey.to_string());
        state
    }

    #[test]
    fn a_capture_lets_go_of_the_hotkey_and_gives_it_back() {
        let state = holding("Alt+Space");
        assert_eq!(state.suspend().as_deref(), Some("Alt+Space"));
        assert!(state.shortcut.lock().unwrap().is_none());
        assert_eq!(state.resume().as_deref(), Some("Alt+Space"));
        assert_eq!(state.resume(), None); // only once
    }

    #[test]
    fn a_second_suspend_keeps_what_the_first_one_let_go_of() {
        let state = holding("Alt+Space");
        state.suspend();
        assert_eq!(state.suspend(), None);
        assert_eq!(state.resume().as_deref(), Some("Alt+Space"));
    }

    #[test]
    fn nothing_is_given_back_when_no_hotkey_was_held() {
        let state = ShortcutState::default();
        assert_eq!(state.suspend(), None);
        assert_eq!(state.resume(), None);
    }

    #[test]
    fn a_hotkey_registered_during_the_capture_is_not_replaced() {
        let state = holding("Alt+Space");
        state.suspend();
        *state.shortcut.lock().unwrap() = Some("Ctrl+Alt+Z".to_string()); // a reload took it
        assert_eq!(state.resume(), None);
    }
}
