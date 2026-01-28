pub mod commands;
pub mod domain;
pub mod infra;
mod services;
pub mod store;

use crate::infra::config::ConfigManager;
use crate::infra::watcher::FileWatcher;
use crate::store::commands::CommandManager;
use crate::store::state::AppState;
use tauri::Manager;
use tauri_plugin_opener;

// エントリーポイント
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = crate::infra::window::show(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Warn)
                .level_for("scoot", log::LevelFilter::Info)
                .level_for("scoot_lib", log::LevelFilter::Info)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
                .max_file_size(50_000) // 50KB
                .targets([
                    // Exeと同じディレクトリの logs フォルダに出力
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder {
                        path: std::env::current_exe()
                            .ok()
                            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                            .unwrap_or_else(|| std::path::PathBuf::from("."))
                            .join("logs"),
                        file_name: None,
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .build(),
        )
        .setup(|app| {
            log::info!("Initializing Scoot");
            // アプリケーション状態(State)の初期化
            let config_manager = ConfigManager::new();
            let mut command_manager = CommandManager::new();
            // Scootコマンドを注入 (Dependency Injection)
            command_manager.set_scoot_commands(crate::services::builtin::get_scoot_commands());

            // Configをロード (非同期)
            let config = tauri::async_runtime::block_on(async { config_manager.load().await })
                .unwrap_or_else(|e| {
                    log::error!("Failed to load initial config: {}", e);
                    crate::domain::config::Config::default()
                });

            let commands_path = config_manager.get_commands_path();
            let config_path = config_manager.get_config_path();
            // ファイルウォッチャー
            let commands_file_watcher = FileWatcher::new(commands_path, app.handle().clone()).ok();
            let config_file_watcher = FileWatcher::new(config_path, app.handle().clone()).ok();

            // State登録
            app.manage(AppState::new(
                command_manager,
                config,
                config_manager,
                commands_file_watcher,
                config_file_watcher,
            ));
            // 共通のデータロード処理を実行
            tauri::async_runtime::block_on(async {
                if let Err(e) = crate::services::system::reload(app.handle()).await {
                    log::error!("Initial configuration load failed: {}", e);
                }
            });
            // ウィンドウイベントの設定
            crate::services::window::setup_window_events(app);
            // イベントリスナーとバックグラウンドタスクの設定
            crate::services::system::setup_event_listeners(app)?;
            crate::services::system::start_bookmark_update_task(app.handle().clone());
            // システムトレイのセットアップ
            crate::services::tray::setup_system_tray(app)?;
            // グローバルショートカットを設定
            crate::services::shortcut::setup_shortcuts(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Command
            commands::command::get_all_commands,
            commands::command::add_command,
            commands::command::update_command,
            commands::command::delete_command,
            commands::command::execute_command,
            commands::command::get_commands_by_prompt,
            commands::command::get_commands,
            commands::command::save_commands,
            commands::command::get_commands_file_path,
            commands::command::get_commands_schema,
            commands::command::validate_commands,
            commands::command::open_commands_json,
            // Config
            commands::config::get_config,
            commands::config::save_config,
            commands::config::get_config_file_path,
            commands::config::get_config_schema,
            commands::config::validate_config,
            commands::config::open_config_json,
            // System
            commands::system::reload_config,
            commands::system::get_file_watcher_status,
            commands::system::open_readme,
            commands::system::quit_app,
            // Window
            commands::window::toggle_window,
            commands::window::hide_window,
            commands::window::show_window,
            commands::window::set_prevent_hide,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
