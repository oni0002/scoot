use tauri::Manager;

/// Reload config and commands
#[tauri::command]
pub async fn reload_all(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    crate::lifecycle::reload(&app_handle).await
}

/// Open README
#[tauri::command]
pub async fn open_readme(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    let resource_path = crate::os::resolve_resource(&app_handle, "README.md")?;
    let path_str = resource_path.to_string_lossy().to_string();
    crate::os::open_path(&app_handle, &path_str)
}

/// Quit app
#[tauri::command]
pub async fn quit_app(app_handle: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    log::info!("Terminating application...");

    for window in app_handle.webview_windows().values() {
        let _ = window.hide();
    }

    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let _ = app_handle.global_shortcut().unregister_all();

    if let Some(state) = app_handle.try_state::<crate::state::AppState>() {
        let commands_config_opt = {
            if let Ok(manager) = state.commands.try_lock() {
                Some(manager.get_user_commands())
            } else {
                None
            }
        };

        if let Some(commands_config) = commands_config_opt {
            let _ = state.command_store.save(&commands_config).await;
        }
    }

    app_handle.exit(0);
    Ok(())
}
