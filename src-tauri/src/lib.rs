pub mod commands;
pub mod config;
pub mod error;
pub mod shortcut;
pub mod state;
pub mod system;
pub mod tray;
pub mod validation;
pub mod watcher;
pub mod window;

use crate::commands::store::{CommandRegistry, CommandStore};
use crate::config::store::ConfigStore;
use crate::state::AppState;
use crate::watcher::FileWatcher;
use tauri::Manager;

// Entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            crate::window::show_main_window(app);
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
            let config_store = ConfigStore::new();
            let mut command_registry = CommandRegistry::new();
            let command_store = CommandStore::new();
            // Inject Scoot commands (Dependency Injection)
            command_registry.set_scoot_commands(crate::commands::domain::get_scoot_commands());

            // Load Config
            let config = tauri::async_runtime::block_on(async { config_store.load().await })
                .unwrap_or_else(|e| {
                    log::error!("Failed to load initial config: {}", e);
                    crate::config::domain::Config::default()
                });

            // config.json watcher
            let config_path = config_store.get_config_path();
            let config_file_watcher = FileWatcher::new(config_path, app.handle().clone()).ok();

            // State generation
            let app_state = AppState::new(
                command_registry,
                command_store,
                config,
                config_store,
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
            crate::system::start_periodic_reload(app.handle().clone());
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
            crate::commands::ipc::open_commands_json,
            crate::commands::ipc::ignore_command,
            // Config
            crate::config::ipc::get_config,
            crate::config::ipc::save_config,
            crate::config::ipc::get_config_file_path,
            crate::config::ipc::open_config_json,
            // System
            crate::system::reload_all,
            crate::system::open_readme,
            crate::system::quit_app,
            // Window
            crate::window::toggle_window,
            crate::window::hide_window,
            crate::window::show_window,
            crate::window::set_prevent_hide,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
