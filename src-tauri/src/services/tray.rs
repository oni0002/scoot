use crate::store::state::AppState;
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

    let mut builder = TrayIconBuilder::with_id("scoot-main")
        .menu(&menu)
        .tooltip("Scoot - Command Launcher")
        .on_menu_event(move |app_handle, event| match event.id().as_ref() {
            "show" => {
                let _ = crate::services::window::show(app_handle);
            }
            "add_command" => {
                let _ = crate::services::system::open_add_command_dialog(app_handle);
            }
            "open_commands" => {
                let _ = crate::services::system::open_commands_json(app_handle);
            }
            "open_config" => {
                let _ = crate::services::system::open_config_json(app_handle);
            }
            "readme" => {
                let _ = crate::services::system::open_readme(app_handle);
            }
            "open_log" => {
                let _ = crate::services::system::open_log_directory(app_handle);
            }
            "quit" => {
                crate::services::system::quit_app(app_handle);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // 左クリックを検出(右クリックは無視)
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app_handle = tray.app_handle();

                if let Some(state) = app_handle.try_state::<AppState>() {
                    // デスクトップでフォーカスが外れて隠れた直後のクリックを無視
                    if let Ok(hidden) = state.last_window_hidden.lock() {
                        if let Some(instant) = *hidden {
                            if instant.elapsed().as_millis() < 300 {
                                log::debug!("tray click ignored due to recent focus loss hide");
                                return;
                            }
                        }
                    }

                    // 300ミリ秒以内の重複クリックを無視
                    if let Ok(mut last_click) = state.last_tray_click.lock() {
                        if let Some(instant) = *last_click {
                            if instant.elapsed().as_millis() < 300 {
                                return;
                            }
                        }
                        *last_click = Some(std::time::Instant::now());
                    }
                }

                // ウィンドウの表示/非表示を切り替える
                if let Err(e) = crate::services::window::toggle_visibility(&app_handle) {
                    log::error!("Failed to toggle window visibility from tray: {}", e);
                }
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let _tray = builder.build(app)?;

    Ok(())
}
