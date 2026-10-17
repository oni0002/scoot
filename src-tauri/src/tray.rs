use crate::state::WindowState;
use tauri::{
    image::Image,
    include_image,
    menu::{MenuBuilder, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager,
};

const TRAY_ID: &str = "scoot-main";

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
    let open_log_item = MenuItem::with_id(app, "open_log", "Open log", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .separator()
        .item(&add_command_item)
        .item(&open_commands_item)
        .item(&open_config_item)
        .item(&open_log_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Scoot - Command Launcher")
        .on_menu_event(move |app_handle, event| match event.id().as_ref() {
            "show" => {
                crate::window::show_main_window(app_handle);
            }
            "add_command" => {
                let _ = crate::lifecycle::open_add_command_dialog(app_handle);
            }
            "open_commands" => {
                let h = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::commands::ipc::open_commands_json(h).await;
                });
            }
            "open_config" => {
                let h = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::config::ipc::open_config_json(h).await;
                });
            }
            "open_log" => {
                let _ = crate::lifecycle::open_log(app_handle);
            }
            "quit" => {
                let h = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::lifecycle::quit_app(h).await;
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

                if let Some(state) = app_handle.try_state::<WindowState>() {
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

                crate::window::toggle_main_window(app_handle);
            }
        });

    let _tray = builder
        .icon(tray_icon(
            app.handle(),
            crate::system_theme::taskbar_is_light(),
        ))
        .build(app)?;

    let handle = app.handle().clone();
    crate::system_theme::watch_taskbar_theme(move |light| {
        if let Some(tray) = handle.tray_by_id(TRAY_ID) {
            if let Err(e) = tray.set_icon(Some(tray_icon(&handle, light))) {
                log::error!("Failed to update the tray icon: {}", e);
            }
        }
    });

    Ok(())
}

/// The single-colour tray icon for the taskbar theme, sized for the display scale
/// (the tray draws icons at 16px x scale).
fn tray_icon(app: &AppHandle, light: bool) -> Image<'static> {
    let scale = app
        .primary_monitor()
        .ok()
        .flatten()
        .map_or(1.0, |m| m.scale_factor());
    let icons = if light {
        [
            include_image!("icons/tray/light-16.png"),
            include_image!("icons/tray/light-20.png"),
            include_image!("icons/tray/light-24.png"),
            include_image!("icons/tray/light-32.png"),
        ]
    } else {
        [
            include_image!("icons/tray/dark-16.png"),
            include_image!("icons/tray/dark-20.png"),
            include_image!("icons/tray/dark-24.png"),
            include_image!("icons/tray/dark-32.png"),
        ]
    };
    let index = match scale {
        s if s <= 1.0 => 0,
        s if s <= 1.25 => 1,
        s if s <= 1.5 => 2,
        _ => 3,
    };
    icons[index].clone()
}
