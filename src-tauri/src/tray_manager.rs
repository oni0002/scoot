use crate::state::AppState;
use crate::window_manager::show_window_with_focus;
use tauri::{
    menu::{MenuBuilder, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    App, Manager,
};

/// システムトレイを設定
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

    let _tray = TrayIconBuilder::with_id("scoot-main")
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Scoot - Command Launcher")
        .on_menu_event(move |app_handle, event| match event.id().as_ref() {
            "show" => {
                let _ = show_window_with_focus(&app_handle);
            }
            "add_command" => {
                let _ = crate::app_setup::open_add_command_dialog(&app_handle);
            }
            "open_commands" => {
                let _ = crate::app_setup::open_commands_json(&app_handle);
            }
            "open_config" => {
                let _ = crate::app_setup::open_config_json(&app_handle);
            }
            "readme" => {
                let _ = crate::app_setup::open_readme();
            }
            "open_log" => {
                let _ = crate::app_setup::open_log_directory(&app_handle);
            }
            "quit" => {
                crate::app_setup::quit_application(&app_handle);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app_handle = tray.app_handle();

                // デバウンス処理: 前回のクリックから一定時間経過していない場合は無視
                if let Some(state) = app_handle.try_state::<AppState>() {
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
                    let is_visible = window.is_visible().unwrap_or(false);
                    if is_visible {
                        let _ = crate::window_manager::hide_window_sync(&app_handle);
                    } else {
                        let _ = show_window_with_focus(&app_handle);
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
