use crate::state::AppState;
use tauri::{
    menu::{MenuBuilder, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    App, Emitter, Manager,
};

/// Setup system tray
pub fn setup_system_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Show Scoot", true, None::<&str>)?;
    let add_command_item =
        MenuItem::with_id(app, "add_command", "Add Command", true, None::<&str>)?;
    let open_commands_item = MenuItem::with_id(
        app,
        "open_commands",
        "Open commands.json",
        true,
        None::<&str>,
    )?;
    let open_config_item =
        MenuItem::with_id(app, "open_config", "Open config.json", true, None::<&str>)?;
    let readme_item = MenuItem::with_id(app, "readme", "Open README", true, None::<&str>)?;
    let open_log_item = MenuItem::with_id(app, "open_log", "Open log", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .separator()
        .item(&add_command_item)
        .item(&open_commands_item)
        .item(&open_config_item)
        .item(&readme_item)
        .item(&open_log_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let mut builder = TrayIconBuilder::with_id("scoot-main")
        .menu(&menu)
        .tooltip("Scoot - Command Launcher")
        .on_menu_event(move |app_handle, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("window-shown", ());

                    // Record the time the window was shown
                    if let Some(state) = app_handle.try_state::<crate::state::AppState>() {
                        if let Ok(mut last_shown) = state.last_window_shown.lock() {
                            *last_shown = Some(std::time::Instant::now());
                        }
                    }
                }
            }
            "add_command" => {
                let _ = crate::system::open_add_command_dialog(app_handle);
            }
            "open_commands" => {
                let _ = crate::system::open_commands_json(app_handle);
            }
            "open_config" => {
                let _ = crate::system::open_config_json(app_handle);
            }
            "readme" => {
                tauri::async_runtime::block_on(async {
                    let _ = crate::system::open_readme(app_handle.clone()).await;
                });
            }
            "open_log" => {
                let _ = crate::system::open_log(app_handle);
            }
            "quit" => {
                tauri::async_runtime::block_on(async {
                    let _ = crate::system::quit_app_command(app_handle.clone()).await;
                });
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Left click detection (right click is ignored)
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app_handle = tray.app_handle();

                if let Some(state) = app_handle.try_state::<AppState>() {
                    // Ignore click if window was hidden due to focus loss
                    if let Ok(hidden) = state.last_window_hidden.lock() {
                        if let Some(instant) = *hidden {
                            if instant.elapsed().as_millis() < 300 {
                                log::debug!("tray click ignored due to recent focus loss hide");
                                return;
                            }
                        }
                    }

                    // Ignore click if within 300ms of last click
                    if let Ok(mut last_click) = state.last_tray_click.lock() {
                        if let Some(instant) = *last_click {
                            if instant.elapsed().as_millis() < 300 {
                                return;
                            }
                        }
                        *last_click = Some(std::time::Instant::now());
                    }
                }

                if let Some(window) = app_handle.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.emit("window-shown", ());

                        // Record the time the window was shown
                        if let Some(state) = app_handle.try_state::<crate::state::AppState>() {
                            if let Ok(mut last_shown) = state.last_window_shown.lock() {
                                *last_shown = Some(std::time::Instant::now());
                            }
                        }
                    }
                }
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let _tray = builder.build(app)?;

    Ok(())
}
