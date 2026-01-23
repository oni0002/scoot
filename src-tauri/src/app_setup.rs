use crate::command_manager::CommandManager;
use crate::config_manager::ConfigManager;
use crate::file_watcher::FileWatcher;
use crate::state::AppState;
use crate::window_manager;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{App, Emitter, Listener, Manager};

/// アプリケーションの初期化
pub fn init(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();
    let last_reload = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(1)));

    // 設定のリロードイベントのリスナー
    app.listen("request-reload", move |_event| {
        log::debug!("Received request-reload event");
        if let Ok(mut last) = last_reload.lock() {
            // デバウンス (500ms未満の場合は無視)
            if last.elapsed() < Duration::from_millis(500) {
                return;
            }
            *last = Instant::now();
        }

        // 非同期で設定をリロード
        let handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = reload_configuration(&handle).await {
                log::error!("Failed to reload configuration: {}", e);
            }
        });
    });

    Ok(())
}

/// アプリケーション状態(State)の初期化
pub fn init_state(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // アプリディレクトリ取得
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    }
    let commands_path = app_dir.join("commands.json");
    let config_path = app_dir.join("config.json");

    // ConfigManagerを初期化
    let config_manager = ConfigManager::new(app_dir.to_string_lossy().to_string());

    // CommandManagerを初期化
    let command_manager = CommandManager::new();

    // ファイルウォッチャー
    let commands_file_watcher = FileWatcher::new(&commands_path, app.handle().clone()).ok();
    let config_file_watcher = FileWatcher::new(&config_path, app.handle().clone()).ok();

    // State管理登録
    app.manage(AppState::new(
        command_manager,
        config_manager,
        commands_file_watcher,
        config_file_watcher,
    ));

    Ok(())
}

/// ウィンドウイベントの設定
pub fn setup_window_events(app: &App) {
    if let Some(window) = app.get_webview_window("main") {
        let window_clone = window.clone();
        window.on_window_event(move |event| {
            match event {
                // ウィンドウのクローズリクエストが送られたとき
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window_clone.hide();
                }
                // フォーカスが変わったとき
                tauri::WindowEvent::Focused(focused) => {
                    window_manager::handle_focused(&window_clone, *focused);
                }
                _ => {}
            }
        });
        // 起動時は非表示
        let _ = window.hide();
    }
}

/// 設定のリロードとアプリケーション状態の更新を行うヘルパー関数
pub async fn reload_configuration(app_handle: &tauri::AppHandle) -> Result<(), String> {
    // Stateの取得
    let state = app_handle
        .try_state::<AppState>()
        .ok_or("Failed to retrieve AppState")?;

    // 設定の読み込み
    log::info!("Loading configuration.");
    let config = state
        .config_manager
        .load_config()
        .map_err(|e| e.to_string())?;

    // コマンドの読み込み
    log::info!("Loading commands.");
    let commands = state
        .config_manager
        .load_commands()
        .map_err(|e| e.to_string())?;

    // ブックマークの読み込み
    log::info!("Loading bookmarks.");
    let bookmarks = if config.bookmarks.enabled {
        crate::bookmark_manager::load_bookmarks(&config.bookmarks)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // アプリケーションスキャン
    log::info!("Loading applications.");
    let app_commands = if config.applications.enabled {
        crate::application_manager::scan_applications(&config.applications.directories)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // CommandManagerに反映 (ロック取得)
    {
        let mut manager = state.command_manager.lock().map_err(|e| e.to_string())?;
        manager.set_user_commands(commands);
        manager.set_bookmark_commands(bookmarks);
        manager.set_application_commands(app_commands);
    }

    // 完了通知
    if let Err(e) = app_handle.emit("config-reloaded", ()) {
        log::error!("Failed to emit config-reloaded: {}", e);
    } else {
        log::info!("Config reloaded successfully, event emitted.");
    }

    Ok(())
}

/// アプリケーションを安全に終了する
pub fn quit_application(app_handle: &tauri::AppHandle) {
    log::info!("Terminating application...");

    // ウィンドウを非表示にする（ユーザーへの即応性のため）
    for window in app_handle.webview_windows().values() {
        let _ = window.hide();
    }

    // グローバルショートカットをクリーンアップ
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let _ = app_handle.global_shortcut().unregister_all();

    // 設定の最終保存を試行
    if let Some(state) = app_handle.try_state::<AppState>() {
        if let Ok(manager) = state.command_manager.try_lock() {
            let commands_config = manager.get_user_commands();
            let _ = state.config_manager.save_commands(&commands_config);
        }
    }

    app_handle.exit(0);
}

/// commands.jsonを開く（存在しない場合は作成）
pub fn open_commands_json(app_handle: &tauri::AppHandle) -> Result<(), String> {
    if let Some(state) = app_handle.try_state::<AppState>() {
        let commands_path = state.config_manager.get_commands_path();

        // ファイルが存在しない場合は作成
        if !std::path::Path::new(&commands_path).exists() {
            let default_commands = crate::models::Commands::default();
            state
                .config_manager
                .save_commands(&default_commands)
                .map_err(|e| e.to_string())?;
        }

        // デフォルトエディタでファイルを開く
        tauri_plugin_opener::open_path(&commands_path, None::<&str>)
            .map_err(|e| format!("Failed to open commands.json: {}", e))?;
    }
    Ok(())
}

/// config.jsonを開く（存在しない場合は作成）
pub fn open_config_json(app_handle: &tauri::AppHandle) -> Result<(), String> {
    if let Some(state) = app_handle.try_state::<AppState>() {
        let config_path = state.config_manager.get_config_path();

        // ファイルが存在しない場合は作成
        if !std::path::Path::new(&config_path).exists() {
            let default_config = crate::models::Config::default();
            state
                .config_manager
                .save_config(&default_config)
                .map_err(|e| e.to_string())?;
        }

        // デフォルトエディタでファイルを開く
        tauri_plugin_opener::open_path(&config_path, None::<&str>)
            .map_err(|e| format!("Failed to open config.json: {}", e))?;
    }
    Ok(())
}

/// README.mdを開く
pub fn open_readme(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let resource_path = app_handle
        .path()
        .resolve("README.md", tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve README.md path: {}", e))?;

    if !resource_path.exists() {
        return Err("README.md not found in resources".to_string());
    }

    let path_str = resource_path.to_string_lossy().to_string();
    tauri_plugin_opener::open_path(&path_str, None::<&str>)
        .map_err(|e| format!("Failed to open README.md: {}", e))
}

/// ログディレクトリを開く
pub fn open_log_directory(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let log_path = app_handle.path().app_log_dir().map_err(|e| e.to_string())?;

    // ディレクトリが存在するか確認し、存在しない場合は作成
    if !log_path.exists() {
        std::fs::create_dir_all(&log_path).map_err(|e| e.to_string())?;
    }

    let path_str = log_path.to_string_lossy().to_string();
    log::info!("Opening log directory: {}", path_str);

    tauri_plugin_opener::open_path(&path_str, None::<&str>)
        .map_err(|e| format!("Failed to open log directory: {}", e))
}

/// コマンド追加ダイアログを開く
pub fn open_add_command_dialog(app_handle: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    app_handle
        .emit("open-add-command-dialog", ())
        .map_err(|e| format!("Failed to emit add command event: {}", e))
}
