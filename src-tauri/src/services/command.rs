use crate::domain::command::{Command, Commands}; // models::Command -> command::Command (Already likely updated but ensuring import is correct or removing duplication if any)
use crate::infra::config::ConfigManager;
use crate::store::commands::CommandManager;

/// コマンド設定(Commands)を取得
pub async fn get_commands(
    config_manager: &ConfigManager,
) -> Result<Commands, crate::domain::error::AppError> {
    config_manager.load_commands().await
}

/// コマンド設定(Commands)を保存
pub async fn save_commands(
    config_manager: &ConfigManager,
    command_manager: &std::sync::Mutex<CommandManager>,
    commands: &Commands,
) -> Result<(), crate::domain::error::AppError> {
    // コマンド設定を保存
    config_manager.save_commands(commands).await?;
    // CommandManagerも更新
    let mut manager = command_manager
        .lock()
        .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
    manager.set_user_commands(commands.clone());
    Ok(())
}

/// コマンドファイルのパスを取得
pub fn get_file_path(config_manager: &ConfigManager) -> String {
    config_manager.get_commands_path().to_string()
}

/// コマンド関連(Commands, Bookmarks, Apps)のみをリロード
pub async fn reload(
    config_manager: &ConfigManager,
    command_manager: &std::sync::Mutex<CommandManager>,
    config: &crate::domain::config::Config,
) -> Result<(), crate::domain::error::AppError> {
    // コマンドの読み込み
    log::info!("Loading commands.");
    let commands = match config_manager.load_commands().await {
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
    let mut manager = command_manager
        .lock()
        .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
    manager.set_user_commands(commands);
    manager.set_bookmark_commands(bookmarks);
    manager.set_application_commands(app_commands);

    Ok(())
}

/// ブックマークのみをリロード
pub async fn reload_bookmarks(
    command_manager: &std::sync::Mutex<CommandManager>,
    config: &crate::domain::config::Config,
) -> Result<(), crate::domain::error::AppError> {
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
    let mut manager = command_manager
        .lock()
        .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
    manager.set_bookmark_commands(bookmarks);

    Ok(())
}

/// 全てのコマンドを取得
pub fn get_all(command_manager: &std::sync::Mutex<CommandManager>) -> Vec<Command> {
    let manager = command_manager.lock().unwrap();
    manager.get_all_commands()
}

/// プロンプトでコマンドを検索
pub fn get_by_prompt(
    command_manager: &std::sync::Mutex<CommandManager>,
    prompt: &str,
) -> Vec<Command> {
    let manager = command_manager.lock().unwrap();
    manager.get_commands_by_prompt(prompt)
}

/// コマンドを追加
pub async fn add(
    config_manager: &ConfigManager,
    command_manager: &std::sync::Mutex<CommandManager>,
    command: Command,
) -> Result<String, crate::domain::error::AppError> {
    let (id, commands) = {
        let mut manager = command_manager
            .lock()
            .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
        // バリデーション
        manager.validate_command(&command)?;

        // コマンド追加
        let id = manager.add_user_command(command);
        let commands = manager.get_user_commands();
        (id, commands)
    };

    // 設定ファイルに保存
    config_manager.save_commands(&commands).await?;

    Ok(id)
}

/// コマンドを更新
pub async fn update(
    config_manager: &ConfigManager,
    command_manager: &std::sync::Mutex<CommandManager>,
    command: Command,
) -> Result<(), crate::domain::error::AppError> {
    let commands = {
        let mut manager = command_manager
            .lock()
            .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
        // バリデーション
        manager.validate_command(&command)?;

        // コマンド更新
        manager.update_user_command(command)?;
        manager.get_user_commands()
    };

    // 設定ファイルに保存
    config_manager.save_commands(&commands).await?;

    Ok(())
}

/// コマンドを削除
pub async fn delete(
    config_manager: &ConfigManager,
    command_manager: &std::sync::Mutex<CommandManager>,
    id: &str,
) -> Result<(), crate::domain::error::AppError> {
    let commands = {
        let mut manager = command_manager
            .lock()
            .map_err(|e| crate::domain::error::AppError::System(e.to_string()))?;
        // コマンド削除
        manager.delete_user_command(id)?;
        manager.get_user_commands()
    };

    // 設定ファイルに保存
    config_manager.save_commands(&commands).await?;

    Ok(())
}
