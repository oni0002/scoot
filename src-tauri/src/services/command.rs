use crate::domain::command::{Command, Commands}; // models::Command -> command::Command (Already likely updated but ensuring import is correct or removing duplication if any)
use crate::store::state::AppState;
use tauri::State;

/// コマンド設定(Commands)を取得
pub async fn get_commands(state: &State<'_, AppState>) -> Result<Commands, String> {
    state.config_manager.load_commands().await
}

/// コマンド設定(Commands)を保存
pub async fn save_commands(state: &State<'_, AppState>, commands: &Commands) -> Result<(), String> {
    // コマンド設定を保存
    state.config_manager.save_commands(commands).await?;
    // CommandManagerも更新
    let mut manager = state.commands.lock().unwrap();
    manager.set_user_commands(commands.clone());
    Ok(())
}

/// コマンドファイルのパスを取得
pub fn get_file_path(state: &State<'_, AppState>) -> String {
    state.config_manager.get_commands_path().to_string()
}

/// コマンド関連(Commands, Bookmarks, Apps)のみをリロード
pub async fn reload(
    app_handle: &tauri::AppHandle,
    config: &crate::domain::config::Config,
) -> Result<(), String> {
    use tauri::Manager;
    let state = app_handle
        .try_state::<AppState>()
        .ok_or("Failed to retrieve AppState")?;

    // コマンドの読み込み
    log::info!("Loading commands.");
    let commands = match state.config_manager.load_commands().await {
        Ok(cmds) => cmds,
        Err(e) => {
            log::error!(
                "Failed to load commands.json: {}. Proceeding with empty commands.",
                e
            );
            crate::domain::command::Commands::new()
        }
    };

    // ブックマークの読み込み
    log::info!("Loading bookmarks.");
    let bookmarks = if config.bookmarks.enabled {
        match crate::infra::bookmark::load(&config.bookmarks).await {
            Ok(bm_commands) => bm_commands,
            Err(e) => {
                log::warn!("Failed to load bookmarks: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // アプリケーションスキャン
    log::info!("Loading applications.");
    let app_commands = if config.applications.enabled {
        crate::infra::application::scan(
            &config.applications.directories,
            &config.applications.extensions,
        )
        .await
        .unwrap_or_default()
    } else {
        Vec::new()
    };

    // CommandManagerに反映
    {
        let mut manager = state.commands.lock().map_err(|e| e.to_string())?;
        manager.set_user_commands(commands);
        manager.set_bookmark_commands(bookmarks);
        manager.set_application_commands(app_commands);
    }

    Ok(())
}

/// ブックマークのみをリロード
pub async fn reload_bookmarks(
    app_handle: &tauri::AppHandle,
    config: &crate::domain::config::Config,
) -> Result<(), String> {
    use tauri::Manager;
    let state = app_handle
        .try_state::<AppState>()
        .ok_or("Failed to retrieve AppState")?;

    // ブックマークの読み込み
    log::info!("Loading bookmarks.");
    let bookmarks = if config.bookmarks.enabled {
        match crate::infra::bookmark::load(&config.bookmarks).await {
            Ok(bm_commands) => bm_commands,
            Err(e) => {
                log::warn!("Failed to load bookmarks: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // CommandManagerに反映
    {
        let mut manager = state.commands.lock().map_err(|e| e.to_string())?;
        manager.set_bookmark_commands(bookmarks);
    }

    Ok(())
}

/// 全てのコマンドを取得
pub fn get_all(state: &State<'_, AppState>) -> Vec<Command> {
    let manager = state.commands.lock().unwrap();
    manager.get_all_commands()
}

/// プロンプトでコマンドを検索
pub fn get_by_prompt(state: &State<'_, AppState>, prompt: &str) -> Vec<Command> {
    let manager = state.commands.lock().unwrap();
    manager.get_commands_by_prompt(prompt)
}

/// コマンドを追加
pub async fn add(state: &State<'_, AppState>, command: Command) -> Result<String, String> {
    let (id, commands) = {
        let mut manager = state.commands.lock().unwrap();

        // バリデーション
        manager.validate_command(&command)?;

        // コマンド追加
        let id = manager.add_user_command(command);
        let commands = manager.get_user_commands();
        (id, commands)
    };

    // 設定ファイルに保存
    state.config_manager.save_commands(&commands).await?;

    Ok(id)
}

/// コマンドを更新
pub async fn update(state: &State<'_, AppState>, command: Command) -> Result<(), String> {
    let commands = {
        let mut manager = state.commands.lock().unwrap();

        // バリデーション
        manager.validate_command(&command)?;

        // コマンド更新
        manager.update_user_command(command)?;
        manager.get_user_commands()
    };

    // 設定ファイルに保存
    state.config_manager.save_commands(&commands).await?;

    Ok(())
}

/// コマンドを削除
pub async fn delete(state: &State<'_, AppState>, id: &str) -> Result<(), String> {
    let commands = {
        let mut manager = state.commands.lock().unwrap();

        // コマンド削除
        manager.delete_user_command(id)?;
        manager.get_user_commands()
    };

    // 設定ファイルに保存
    state.config_manager.save_commands(&commands).await?;

    Ok(())
}
