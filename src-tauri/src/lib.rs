mod app_setup;
mod application_manager;
pub mod bookmark_manager;
mod command_executor;

pub mod command_manager;
mod config_manager;
mod file_watcher;
pub mod models;
mod shortcut_manager;
pub mod state;
mod tray_manager;
pub mod utils;
pub mod window_manager;

use models::Command;
use state::AppState;

use tauri::State;

use tauri_plugin_opener;

/// tauri command: すべてのコマンドを取得
#[tauri::command]
async fn get_all_commands(state: State<'_, AppState>) -> Result<Vec<Command>, String> {
    let manager = state.command_manager.lock().unwrap();
    Ok(manager.get_all_commands())
}

/// tauri command: コマンドを追加
#[tauri::command]
async fn add_command(command: Command, state: State<'_, AppState>) -> Result<String, String> {
    let mut manager = state.command_manager.lock().unwrap();

    // バリデーション
    manager.validate_command(&command)?;

    // コマンド追加
    let id = manager.add_command(command);
    drop(manager); // ロックを早期解放

    // 設定ファイルに保存
    state.save_config()?;

    Ok(id)
}

/// tauri command: コマンドを更新
#[tauri::command]
async fn update_command(command: Command, state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.command_manager.lock().unwrap();

    // バリデーション
    manager.validate_command(&command)?;

    // コマンド更新
    manager.update_command(command)?;
    drop(manager); // ロックを早期解放

    // 設定ファイルに保存
    state.save_config()?;

    Ok(())
}

/// tauri command: コマンドを削除
#[tauri::command]
async fn delete_command(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.command_manager.lock().unwrap();

    // コマンド削除
    manager.delete_command(&id)?;
    // ロックを早期解放
    drop(manager);

    // 設定ファイルに保存
    state.save_config()?;

    Ok(())
}

/// tauri command: コマンドを実行(非同期)
#[tauri::command]
async fn execute_command(
    command: Command,
    args: Vec<String>,
    app_handle: tauri::AppHandle,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    crate::command_executor::execute_command(&app_handle, &command, &args).await
}

/// tauri command: プロンプトでコマンドを検索
#[tauri::command]
async fn get_commands_by_prompt(
    prompt: String,
    state: State<'_, AppState>,
) -> Result<Vec<Command>, String> {
    let manager = state.command_manager.lock().unwrap();
    Ok(manager.get_commands_by_prompt(&prompt))
}

/// tauri command: 設定、コマンドをリロード
#[tauri::command]
async fn reload_config(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::app_setup::reload_configuration(&app_handle).await
}

/// tauri command: 設定を取得
#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<crate::models::Config, String> {
    state.config_manager.load_config()
}

/// tauri command: 設定を保存
#[tauri::command]
async fn save_config(
    config: crate::models::Config,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.config_manager.save_config(&config)
}

/// tauri command: コマンドを取得
#[tauri::command]
async fn get_commands(state: State<'_, AppState>) -> Result<crate::models::Commands, String> {
    state.config_manager.load_commands()
}

/// tauri command: コマンドを保存
#[tauri::command]
async fn save_commands(
    commands: crate::models::Commands,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // コマンド設定を保存
    state.config_manager.save_commands(&commands)?;
    // CommandManagerも更新
    let mut manager = state.command_manager.lock().unwrap();
    manager.set_user_commands(commands);
    Ok(())
}

/// tauri command: config.jsonのパスを取得
#[tauri::command]
async fn get_config_file_path(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.config_manager.get_config_path().to_string())
}

/// tauri command: commands.jsonのパスを取得
#[tauri::command]
async fn get_commands_file_path(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.config_manager.get_commands_path().to_string())
}

/// tauri command: config.jsonのスキーマを取得
#[tauri::command]
async fn get_config_schema() -> Result<serde_json::Value, String> {
    Ok(models::Config::generate_schema())
}

/// tauri command: commands.jsonのスキーマを取得
#[tauri::command]
async fn get_commands_schema() -> Result<serde_json::Value, String> {
    Ok(models::generate_commands_schema())
}

/// tauri command: config.jsonを検証
#[tauri::command]
async fn validate_config(config: serde_json::Value) -> Result<serde_json::Value, String> {
    match models::Config::from_json_with_validation(&config.to_string()) {
        Ok(_) => Ok(serde_json::json!({ "valid": true, "errors": [] })),
        Err(error) => Ok(serde_json::json!({ "valid": false, "errors": [error] })),
    }
}

/// tauri command: commands.jsonを検証
#[tauri::command]
async fn validate_commands(config: serde_json::Value) -> Result<serde_json::Value, String> {
    match models::commands_from_json_with_validation(&config.to_string()) {
        Ok(_) => Ok(serde_json::json!({ "valid": true, "errors": [] })),
        Err(error) => Ok(serde_json::json!({ "valid": false, "errors": [error] })),
    }
}

/// tauri command: ファイルウォッチャーの状態を取得
#[tauri::command]
async fn get_file_watcher_status(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state._commands_file_watcher.is_some() || state._app_config_file_watcher.is_some())
}

/// tauri command: commands.jsonを開く
#[tauri::command]
async fn open_commands_json(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::app_setup::open_commands_json(&app_handle)
}

/// tauri command: config.jsonを開く
#[tauri::command]
async fn open_config_json(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::app_setup::open_config_json(&app_handle)
}

/// tauri command: READMEを開く
#[tauri::command]
async fn open_readme() -> Result<(), String> {
    crate::app_setup::open_readme()
}

/// tauri command: アプリを終了
#[tauri::command]
async fn quit_app(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::app_setup::quit_application(&app_handle);
    Ok(())
}

/// tauri command: ウィンドウを表示/非表示を切り替える
#[tauri::command]
async fn toggle_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::window_manager::toggle_window_visibility(&app_handle)
}

/// tauri command: ウィンドウを隠す
#[tauri::command]
async fn hide_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::window_manager::hide_window_sync(&app_handle)
}

/// tauri command: ウィンドウを表示
#[tauri::command]
async fn show_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::window_manager::show_window_with_focus(&app_handle)
}

/// tauri command: prevent_hideフラグを設定
#[tauri::command]
async fn set_prevent_hide(prevent: bool, state: State<'_, AppState>) -> Result<(), String> {
    crate::window_manager::set_prevent_hide_flag(prevent, &state)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Warn)
                .level_for("scoot", log::LevelFilter::Info)
                .level_for("scoot_lib", log::LevelFilter::Info)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
                .max_file_size(50_000) // 50KB
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: None,
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .build(),
        )
        .setup(|app| {
            // アプリケーション状態(State)の初期化
            app_setup::init_state(app)?;
            // 共通のデータロード処理を実行
            tauri::async_runtime::block_on(async {
                if let Err(e) = app_setup::reload_configuration(app.handle()).await {
                    log::error!("Initial configuration load failed: {}", e);
                }
            });
            // ウィンドウイベントの設定
            app_setup::setup_window_events(app);
            // ファイル変更イベントのリスナー初期化 (app_setup.rs)
            app_setup::init(app)?;
            // システムトレイのセットアップ
            tray_manager::setup_system_tray(app)?;
            // グローバルショートカットを設定
            shortcut_manager::setup_global_shortcuts(app)?;
            log::info!("Scoot initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_all_commands,
            add_command,
            update_command,
            delete_command,
            execute_command,
            get_commands_by_prompt,
            reload_config,
            get_config,
            save_config,
            get_commands,
            save_commands,
            get_file_watcher_status,
            get_config_file_path,
            get_commands_file_path,
            get_config_schema,
            get_commands_schema,
            validate_config,
            validate_commands,
            toggle_window,
            hide_window,
            show_window,
            open_commands_json,
            open_config_json,
            open_readme,
            set_prevent_hide,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
