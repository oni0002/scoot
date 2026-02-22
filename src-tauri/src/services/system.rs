use crate::store::state::AppState;
use tauri::{AppHandle, Manager, State};

/// アプリを終了
pub fn quit_app(app_handle: &AppHandle) {
    log::info!("Terminating application...");
    use tauri::Manager;

    // ウィンドウを非表示にする（ユーザーへの即応性のため）
    for window in app_handle.webview_windows().values() {
        let _ = window.hide();
    }

    // グローバルショートカットをクリーンアップ
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let _ = app_handle.global_shortcut().unregister_all();

    // 設定の最終保存を試行
    if let Some(state) = app_handle.try_state::<AppState>() {
        // try_lockを使用（デッドロック回避）
        if let Ok(manager) = state.commands.try_lock() {
            let commands_config = manager.get_user_commands();
            let _ = tauri::async_runtime::block_on(async {
                state.config_manager.save_commands(&commands_config).await
            });
        }
    }

    app_handle.exit(0);
}

/// Config, Commandsをリロード
pub async fn reload(app_handle: &tauri::AppHandle) -> Result<(), crate::domain::error::AppError> {
    use tauri::Emitter;

    // Configリロード (Configオブジェクトを受け取る)
    let config = if let Some(state) = app_handle.try_state::<AppState>() {
        let new_config =
            crate::services::config::reload(&state.config_manager, &state.config).await?;
        // ホットキーを再登録
        if let Err(e) =
            crate::services::shortcut::setup_global_shortcuts(app_handle, &new_config.hotkey)
        {
            log::warn!("Failed to re-register hotkey: {}", e);
        }
        new_config
    } else {
        return Err(crate::domain::error::AppError::System(
            "Failed to get AppState".to_string(),
        ));
    };

    // コマンドリロード (Configを渡す)
    if let Some(state) = app_handle.try_state::<AppState>() {
        crate::services::command::reload(&state.config_manager, &state.commands, &config).await?;
    } else {
        return Err(crate::domain::error::AppError::System(
            "Failed to get AppState".to_string(),
        ));
    }

    if let Err(e) = app_handle.emit("config-reloaded", ()) {
        log::error!("Failed to emit config-reloaded: {}", e);
    } else {
        log::info!("Config reloaded successfully, event emitted.");
    }

    Ok(())
}

/// commands.jsonを開く
pub fn open_commands_json(app_handle: &AppHandle) -> Result<(), crate::domain::error::AppError> {
    if let Some(state) = app_handle.try_state::<AppState>() {
        let commands_path = state.config_manager.get_commands_path();

        crate::infra::system::ensure_file_exists(std::path::Path::new(&commands_path), || {
            let default_commands = crate::domain::command::Commands::default();
            tauri::async_runtime::block_on(async {
                state
                    .config_manager
                    .save_commands(&default_commands)
                    .await
                    .map_err(|e| crate::domain::error::AppError::System(e.to_string()))
            })
        })?;

        crate::infra::system::open_path(app_handle, &commands_path)?;
    }
    Ok(())
}

/// config.jsonを開く
pub fn open_config_json(app_handle: &AppHandle) -> Result<(), crate::domain::error::AppError> {
    if let Some(state) = app_handle.try_state::<AppState>() {
        let config_path = state.config_manager.get_config_path();

        crate::infra::system::ensure_file_exists(std::path::Path::new(&config_path), || {
            let default_config = crate::domain::config::Config::default();
            tauri::async_runtime::block_on(async {
                state
                    .config_manager
                    .save(&default_config)
                    .await
                    .map_err(|e| crate::domain::error::AppError::System(e.to_string()))
            })
        })?;

        crate::infra::system::open_path(app_handle, &config_path)?;
    }
    Ok(())
}

/// READMEを開く
pub fn open_readme(app_handle: &AppHandle) -> Result<(), crate::domain::error::AppError> {
    let resource_path = crate::infra::system::resolve_resource(app_handle, "README.md")?;
    let path_str = resource_path.to_string_lossy().to_string();
    crate::infra::system::open_path(app_handle, &path_str)
}

/// ファイルウォッチャーの状態を取得
pub fn get_file_watcher_status(state: &State<'_, AppState>) -> bool {
    state._commands_file_watcher.is_some() || state._config_file_watcher.is_some()
}

/// ログ(またはログディレクトリ)を開く
pub fn open_log(app_handle: &AppHandle) -> Result<(), crate::domain::error::AppError> {
    let log_dir = crate::infra::system::get_log_dir(app_handle)?;
    crate::infra::system::ensure_directory_exists(&log_dir)?;

    let log_file = log_dir.join("scoot.log");
    let target_path = if log_file.exists() { log_file } else { log_dir };

    let path_str = target_path.to_string_lossy().to_string();
    log::debug!("Opening log path: {}", path_str);
    crate::infra::system::open_path(app_handle, &path_str)
}

/// コマンド追加ダイアログを開く
pub fn open_add_command_dialog(
    app_handle: &AppHandle,
) -> Result<(), crate::domain::error::AppError> {
    use tauri::{Emitter, Manager};
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    app_handle.emit("open-add-command-dialog", ()).map_err(|e| {
        crate::domain::error::AppError::System(format!("Failed to emit add command event: {}", e))
    })
}

/// ブックマーク自動更新タスクを開始
pub fn start_bookmark_update_task(app_handle: AppHandle) {
    // use tauri::Manager; // This is now at file scope
    tauri::async_runtime::spawn(async move {
        log::debug!("Starting bookmark auto-refresh task");
        loop {
            // 現在の設定からリフレッシュ間隔を取得
            let interval_minutes = if let Some(state) = app_handle.try_state::<AppState>() {
                if let Ok(config) = state.config.lock() {
                    // 最小値制限 (1分)
                    std::cmp::max(config.bookmarks.refresh_interval_minutes, 1)
                } else {
                    30 // ロック取得失敗時
                }
            } else {
                30 // State取得失敗時
            };

            // 指定時間待機
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_minutes * 60)).await;

            log::debug!("Executing scheduled bookmark refresh...");

            // Configを取得してリロード
            let config_opt = if let Some(state) = app_handle.try_state::<AppState>() {
                state.config.lock().ok().map(|c| c.clone())
            } else {
                None
            };

            if let Some(config) = config_opt {
                // ブックマークのみリロード
                if let Some(state) = app_handle.try_state::<AppState>() {
                    if let Err(e) =
                        crate::services::command::reload_bookmarks(&state.commands, &config).await
                    {
                        log::error!("Failed to auto-refresh bookmarks: {}", e);
                    }
                }
            } else {
                log::warn!("Skipping bookmark refresh due to failure in retrieving AppState or Config lock");
            }
        }
    });
}

/// イベントリスナーを設定
pub fn setup_event_listeners(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    use tauri::{Emitter, Listener};

    let app_handle = app.handle().clone();
    let last_reload = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(1)));

    // 設定リロードイベントを受けると設定をリロードする
    app.listen("request-reload", move |_event| {
        log::debug!("Received request-reload event");
        if let Ok(mut last) = last_reload.lock() {
            // 500ms未満の場合は無視 (デバウンス)
            if last.elapsed() < Duration::from_millis(500) {
                return;
            }
            *last = Instant::now();
        }

        // 非同期で設定をリロード
        let handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = crate::services::system::reload(&handle).await {
                log::error!("Failed to reload configuration: {}", e);
            }
        });
    });

    // ファイル変更イベントをリロードリクエストに転送
    let app_handle_for_proxy = app.handle().clone();
    app.listen("config-file-changed", move |_| {
        let _ = app_handle_for_proxy.emit("request-reload", ());
    });

    Ok(())
}
