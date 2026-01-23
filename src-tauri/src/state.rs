use crate::command_manager::CommandManager;
use crate::config_manager::ConfigManager;
use crate::file_watcher::FileWatcher;
use std::sync::Mutex;

pub struct AppState {
    pub command_manager: Mutex<CommandManager>,
    pub config_manager: ConfigManager,

    // Pub(crate) allow access within the crate
    pub _commands_file_watcher: Option<FileWatcher>,
    pub _app_config_file_watcher: Option<FileWatcher>,
    pub registered_shortcut: Mutex<Option<String>>,
    // ウィンドウが表示された最後の時刻（Trayアイコンクリック時の即時非表示を防ぐため）
    pub last_window_shown: Mutex<Option<std::time::Instant>>,
    // トレイアイコンがクリックされた最後の時刻（ダブル発火防止のため）
    pub last_tray_click: Mutex<Option<std::time::Instant>>,
    // フォーカスが外れてもウィンドウを隠さないフラグ（ファイルダイアログ表示用など）
    pub prevent_hide: Mutex<bool>,
}

impl AppState {
    pub fn new(
        command_manager: CommandManager,
        config_manager: ConfigManager,
        commands_file_watcher: Option<FileWatcher>,
        app_config_file_watcher: Option<FileWatcher>,
    ) -> Self {
        Self {
            command_manager: Mutex::new(command_manager),
            config_manager,
            _commands_file_watcher: commands_file_watcher,
            _app_config_file_watcher: app_config_file_watcher,
            registered_shortcut: Mutex::new(None),
            last_window_shown: Mutex::new(None),
            last_tray_click: Mutex::new(None),
            prevent_hide: Mutex::new(false),
        }
    }

    /// 設定を保存する共通メソッド
    pub fn save_config(&self) -> Result<(), String> {
        let manager = self.command_manager.lock().unwrap();
        let commands = manager.get_user_commands();
        self.config_manager.save_commands(&commands)
    }
}
