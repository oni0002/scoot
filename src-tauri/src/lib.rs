pub mod commands;
pub mod config;
pub mod error;
pub mod shortcut;
pub mod state;
pub mod system;
pub mod tray;
pub mod watcher;
pub mod window;

use crate::commands::store::CommandManager;
use crate::config::store::ConfigManager;
use crate::state::AppState;
use crate::watcher::FileWatcher;
use tauri::{Emitter, Manager};

// Entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.emit("window-shown", ());

                if let Some(state) = app.try_state::<AppState>() {
                    if let Ok(mut last_shown) = state.last_window_shown.lock() {
                        *last_shown = Some(std::time::Instant::now());
                    }
                }
            }
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
                    // Output to logs folder in the same directory as the exe
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
            // Initialize application state
            let config_manager = ConfigManager::new();
            let mut command_manager = CommandManager::new();
            // Inject Scoot commands (Dependency Injection)
            command_manager.set_scoot_commands(crate::commands::domain::get_builtin_commands());

            // Load Config (async)
            let config = tauri::async_runtime::block_on(async { config_manager.load().await })
                .unwrap_or_else(|e| {
                    log::error!("Failed to load initial config: {}", e);
                    crate::config::domain::Config::default()
                });

            let commands_path = config_manager.get_commands_path();
            let config_path = config_manager.get_config_path();
            // File watchers
            let commands_file_watcher = FileWatcher::new(commands_path, app.handle().clone()).ok();
            let config_file_watcher = FileWatcher::new(config_path, app.handle().clone()).ok();

            // State generation
            let app_state = AppState::new(
                command_manager,
                config,
                config_manager,
                commands_file_watcher,
                config_file_watcher,
            );

            // Clone Arcs for window event listeners
            let last_window_shown = app_state.last_window_shown.clone();
            let prevent_hide = app_state.prevent_hide.clone();
            let last_window_hidden = app_state.last_window_hidden.clone();

            // State registration
            app.manage(app_state);

            // Run common data loading process
            tauri::async_runtime::block_on(async {
                if let Err(e) = crate::system::reload(app.handle()).await {
                    log::error!("Initial configuration load failed: {}", e);
                }
            });
            // Set window events
            crate::window::setup_window_events(
                app,
                last_window_shown,
                prevent_hide,
                last_window_hidden,
            );
            // Set event listeners and background tasks
            crate::system::setup_event_listeners(app)?;
            crate::system::start_bookmark_update_task(app.handle().clone());
            // Set up system tray
            crate::tray::setup_system_tray(app)?;
            // Set up global shortcuts
            crate::shortcut::setup_shortcuts(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Command
            crate::commands::ipc::get_all_commands,
            crate::commands::ipc::add_command,
            crate::commands::ipc::update_command,
            crate::commands::ipc::delete_command,
            crate::commands::execution::execute_command,
            crate::commands::ipc::get_commands_by_prompt,
            crate::commands::ipc::get_commands,
            crate::commands::ipc::save_commands,
            crate::commands::ipc::get_commands_file_path,
            crate::commands::ipc::get_commands_schema,
            crate::commands::ipc::validate_commands,
            crate::commands::ipc::open_commands_json,
            // Config
            crate::config::ipc::get_config,
            crate::config::ipc::save_config,
            crate::config::ipc::get_config_file_path,
            crate::config::ipc::get_config_schema,
            crate::config::ipc::validate_config,
            crate::config::ipc::open_config_json,
            // System
            crate::system::reload_config,
            crate::system::get_file_watcher_status,
            crate::system::open_readme,
            crate::system::quit_app_command,
            // Window
            crate::window::toggle_window,
            crate::window::set_prevent_hide,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
