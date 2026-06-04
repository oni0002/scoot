pub mod commands;
pub mod config;
pub mod error;
pub mod lifecycle;
pub mod os;
pub mod shortcut;
pub mod state;
pub mod tray;
pub mod validation;
pub mod watcher;
pub mod window;

use crate::commands::registry::CommandRegistry;
use crate::commands::store::CommandStore;
use crate::config::store::ConfigStore;
use crate::state::{CommandsState, ConfigState, ShortcutState, WindowState};
use crate::watcher::FileWatcher;
use tauri::Manager;

// Entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
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
            let command_registry = CommandRegistry::new();
            let command_store = CommandStore::new();

            // Load Config
            let config = tauri::async_runtime::block_on(async { config_store.load().await })
                .unwrap_or_else(|e| {
                    log::error!("Failed to load initial config: {}", e);
                    crate::config::domain::Config::default()
                });

            // config.json watcher
            let config_path = config_store.get_config_path();
            let config_file_watcher = FileWatcher::new(config_path, app.handle().clone()).ok();

            // State construction
            let commands_state = CommandsState::new(command_registry, command_store);
            let config_state = ConfigState::new(config, config_store, config_file_watcher);
            let shortcut_state = ShortcutState::new();
            let window_state = WindowState::new();

            // Clone Arcs for window event listeners before moving into managed state
            let last_window_shown = window_state.last_window_shown.clone();
            let prevent_hide = window_state.prevent_hide.clone();
            let last_window_hidden = window_state.last_window_hidden.clone();

            // State registration
            app.manage(commands_state);
            app.manage(config_state);
            app.manage(shortcut_state);
            app.manage(window_state);

            // Run common data loading process
            tauri::async_runtime::block_on(async {
                if let Err(e) = crate::lifecycle::reload(app.handle()).await {
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
            crate::lifecycle::setup_reload_listeners(app)?;
            crate::lifecycle::start_periodic_reload(app.handle().clone());
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
            crate::commands::ipc::get_commands_by_alias,
            crate::commands::ipc::open_commands_json,
            // Config
            crate::config::ipc::get_config,
            crate::config::ipc::save_config,
            crate::config::ipc::ignore_command,
            crate::config::ipc::open_config_json,
            // Lifecycle
            crate::lifecycle::reload_all,
            crate::lifecycle::open_readme,
            crate::lifecycle::quit_app,
            // Window
            crate::window::hide_window,
            crate::window::enter_modal,
            crate::window::leave_modal,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            log::error!("Fatal: failed to run tauri application: {}", e);
            eprintln!("Fatal: failed to run tauri application: {e}");
            std::process::exit(1);
        });
}
