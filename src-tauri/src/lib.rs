pub mod commands;
pub mod error;
pub mod hotkey_capture;
pub mod json;
pub mod lifecycle;
pub mod notices;
pub mod os;
pub mod settings;
pub mod shortcut;
pub mod startup;
pub mod state;
pub mod system_theme;
pub mod tray;
pub mod watcher;
pub mod window;

use crate::commands::registry::CommandRegistry;
use crate::commands::store::CommandStore;
use crate::settings::store::SettingsStore;
use crate::state::{CommandsState, SettingsState, ShortcutState, WindowState};
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
                        path: crate::os::log_dir(),
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
            let settings_store = SettingsStore::new();
            let command_registry = CommandRegistry::default();
            let command_store = CommandStore::new();

            // Load Settings
            let settings = tauri::async_runtime::block_on(async { settings_store.load().await })
                .unwrap_or_else(|e| {
                    log::error!("Failed to load initial settings: {}", e);
                    crate::settings::domain::Settings::default()
                });

            // settings.json watcher
            let settings_file_watcher =
                FileWatcher::new(&settings_store.watched_paths(), app.handle().clone())
                    .map_err(|e| {
                        log::warn!(
                            "Edits to settings.json will only be picked up by the next Reload: {}",
                            e
                        )
                    })
                    .ok();

            // State construction
            let commands_state = CommandsState::new(command_registry, command_store);
            let settings_state =
                SettingsState::new(settings, settings_store, settings_file_watcher);
            let shortcut_state = ShortcutState::default();
            let notice_state = crate::notices::NoticeState::default();
            let window_state = WindowState::default();

            // State registration
            app.manage(commands_state);
            app.manage(settings_state);
            app.manage(shortcut_state);
            app.manage(notice_state);
            app.manage(window_state);

            // Set window events
            crate::window::setup_window_events(app);
            // Set event listeners and background tasks
            crate::lifecycle::setup_reload_listeners(app)?;
            crate::lifecycle::start_periodic_reload(app.handle().clone());
            // Set up system tray
            crate::tray::setup_system_tray(app)?;
            // Set up global shortcuts. A Hotkey another app holds must not stop Scoot from starting:
            // the tray still opens it, and the page reports the failure once it is up.
            if let Err(e) = crate::shortcut::setup_shortcuts(app) {
                log::warn!("Starting without a Hotkey: {}", e);
            }
            // Install window subclass to suppress WM_SYSCOMMAND SC_KEYMENU
            // during hotkey capture (prevents Alt+Space from opening system menu).
            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(hwnd) = window.hwnd() {
                    unsafe {
                        crate::hotkey_capture::setup_window_subclass(hwnd.0);
                    }
                }
            }
            // Load the command sources in the background: scanning them must not delay the
            // hotkey and tray. The webview refetches when `settings-reloaded` arrives.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::lifecycle::reload(&handle).await {
                    log::error!("Initial configuration load failed: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Command
            crate::commands::ipc::get_all_commands,
            crate::commands::ipc::add_command,
            crate::commands::ipc::update_command,
            crate::commands::ipc::delete_command,
            crate::commands::execution::execute_command,
            crate::commands::execution::open_target,
            crate::commands::usage::get_usage,
            crate::commands::usage::clear_usage,
            crate::commands::ipc::open_commands_json,
            // Settings
            crate::settings::ipc::get_settings,
            crate::settings::ipc::save_settings,
            crate::settings::ipc::ignore_command,
            crate::settings::ipc::unignore_command,
            crate::settings::ipc::set_external_alias,
            crate::settings::ipc::open_config_json,
            // Lifecycle
            crate::lifecycle::reload_all,
            // Window
            crate::window::hide_window,
            crate::window::enter_modal,
            crate::window::leave_modal,
            // Startup
            crate::notices::get_notices,
            crate::startup::get_startup_shortcut,
            crate::startup::set_startup_shortcut,
            // Hotkey capture
            crate::hotkey_capture::start_hotkey_capture,
            crate::hotkey_capture::stop_hotkey_capture,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            log::error!("Fatal: failed to run tauri application: {}", e);
            eprintln!("Fatal: failed to run tauri application: {e}");
            std::process::exit(1);
        });
}
