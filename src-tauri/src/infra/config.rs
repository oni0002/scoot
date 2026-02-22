use crate::domain::command::Commands;
use crate::domain::config::Config;
use serde_json;
#[allow(unused_imports)]
use std::fs; // Keep for now if needed, but we are switching to tokio

pub struct ConfigManager {
    config_path: String,   // 設定ファイルのパス
    commands_path: String, // コマンドファイルのパス
}

/// Configを管理するクラス
/// config.jsonとcommands.jsonのバリデーション、シリアライズ/デシリアライズ、デフォルトの生成を行う
impl ConfigManager {
    pub fn new() -> Self {
        // 実行ディレクトリを取得
        let target_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        log::debug!("Using config: {}", target_dir);

        let config_path = format!("{}/config.json", target_dir);
        let commands_path = format!("{}/commands.json", target_dir);

        Self {
            config_path,
            commands_path,
        }
    }

    /// Configを読み込む
    pub async fn load(&self) -> Result<Config, crate::domain::error::AppError> {
        // config.jsonが存在しない場合、デフォルト値を保存して返す
        if !tokio::fs::try_exists(&self.config_path)
            .await
            .unwrap_or(false)
        {
            let default_config = Config::default();
            self.save(&default_config).await?;
            return Ok(default_config);
        }

        // config.jsonを読み込む
        let content = tokio::fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| {
                crate::domain::error::AppError::System(format!(
                    "Failed to read app config file '{}': {}",
                    self.config_path, e
                ))
            })?;

        // config.jsonが空の場合、デフォルト値を保存して返す
        if content.trim().is_empty() {
            let default_config = Config::default();
            self.save(&default_config).await?;
            return Ok(default_config);
        }

        // config.jsonをパースして返す
        let mut config: Config = Config::from_json_with_validation(&content).map_err(|e| {
            crate::domain::error::AppError::System(format!(
                "App config file '{}' validation failed: {}",
                self.config_path, e
            ))
        })?;

        // fuzzy_thresholdを検証して必要なら修正
        config.validate_and_fix()?;

        Ok(config)
    }

    /// Commandsを読み込む
    pub async fn load_commands(&self) -> Result<Commands, crate::domain::error::AppError> {
        // commands.jsonが存在しない場合、デフォルト値を保存して返す
        if !tokio::fs::try_exists(&self.commands_path)
            .await
            .unwrap_or(false)
        {
            let default_commands: Commands = Vec::new();
            self.save_commands(&default_commands).await?;
            return Ok(default_commands);
        }

        // commands.jsonを読み込む
        let content = tokio::fs::read_to_string(&self.commands_path)
            .await
            .map_err(|e| {
                crate::domain::error::AppError::System(format!(
                    "Failed to read commands file '{}': {}",
                    self.commands_path, e
                ))
            })?;

        // commands.jsonが空の場合、デフォルト値を保存して返す
        if content.trim().is_empty() {
            let default_commands: Commands = Vec::new();
            self.save_commands(&default_commands).await?;
            return Ok(default_commands);
        }

        // commands.jsonを検証してパース
        // JSONパースは重い処理の可能性があるためspawn_blockingで実行
        let commands = tokio::task::spawn_blocking(move || {
            crate::domain::config::commands_from_json_with_validation(&content)
        })
        .await
        .map_err(|e| {
            crate::domain::error::AppError::System(format!("Failed to spawn blocking task: {}", e))
        })?
        .unwrap_or_else(|e| {
            log::error!("Failed to parse commands.json: {}", e);
            Vec::new()
        });

        Ok(commands)
    }

    /// Configを保存
    pub async fn save(
        &self,
        config: &crate::domain::config::Config,
    ) -> Result<(), crate::domain::error::AppError> {
        self.save_to_json(&self.config_path, config).await
    }

    /// Commandsを保存
    pub async fn save_commands(
        &self,
        commands: &crate::domain::command::Commands,
    ) -> Result<(), crate::domain::error::AppError> {
        self.save_to_json(&self.commands_path, commands).await
    }

    /// JSONで保存
    async fn save_to_json<T: serde::Serialize>(
        &self,
        path: &str,
        data: &T,
    ) -> Result<(), crate::domain::error::AppError> {
        let content = serde_json::to_string_pretty(data).map_err(|e| {
            crate::domain::error::AppError::System(format!("Failed to serialize data: {}", e))
        })?;

        tokio::fs::write(path, content).await.map_err(|e| {
            crate::domain::error::AppError::System(format!("Failed to write to file: {}", e))
        })
    }

    /// 設定ファイルのパスを取得
    pub fn get_config_path(&self) -> &str {
        &self.config_path
    }

    /// コマンドファイルのパスを取得
    pub fn get_commands_path(&self) -> &str {
        &self.commands_path
    }
}
