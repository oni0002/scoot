use crate::domain::config::Config;
use crate::store::state::AppState;
use tauri::State;

/// 設定を取得
pub fn get(state: &State<'_, AppState>) -> Result<Config, String> {
    state
        .config
        .lock()
        .map(|c| c.clone())
        .map_err(|e| e.to_string())
}

/// 設定を保存
pub async fn save(state: &State<'_, AppState>, config: &Config) -> Result<(), String> {
    // stateを更新
    {
        let mut state_config = state.config.lock().map_err(|e| e.to_string())?;
        *state_config = config.clone();
    }

    // config.jsonに保存
    state.config_manager.save(config).await
}

/// 設定ファイルのパスを取得
pub fn get_file_path(state: &State<'_, AppState>) -> String {
    state.config_manager.get_config_path().to_string()
}

/// 設定をリロード
pub async fn reload(app_handle: &tauri::AppHandle) -> Result<Config, String> {
    use tauri::Manager;

    // Stateの取得
    let state = app_handle
        .try_state::<AppState>()
        .ok_or("Failed to retrieve AppState")?;

    // 設定の読み込み
    log::info!("Loading configuration.");
    let new_config = state
        .config_manager
        .load()
        .await
        .map_err(|e| e.to_string())?;

    // メモリ上のConfigを更新
    {
        let mut state_config = state.config.lock().map_err(|e| e.to_string())?;
        *state_config = new_config.clone();
    }

    // ホットキーを再登録
    if let Err(e) =
        crate::services::shortcut::setup_global_shortcuts(app_handle, &new_config.hotkey)
    {
        log::warn!("Failed to re-register hotkey: {}", e);
        // エラーでもリロード処理自体は成功とする（警告のみ）
    }

    Ok(new_config)
}
