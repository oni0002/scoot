use crate::models::{Commands, Config};
use serde_json;
use std::fs;
use std::path::Path;

/// 設定とコマンドを分離して管理するConfigManager
pub struct ConfigManager {
    config_path: String,
    commands_path: String,
}

impl ConfigManager {
    /// 新しいConfigManagerを作成
    pub fn new(app_data_dir: String) -> Self {
        let mut target_dir = app_data_dir.clone();
        let mut is_portable = false;

        // 実行ファイルのパスを取得
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let exe_dir_str = exe_dir.to_string_lossy().to_string();
                let portable_config = exe_dir.join("config.json");
                let installed_config = Path::new(&app_data_dir).join("config.json");

                // ポータブル設定が既に存在する -> ポータブル使用
                if portable_config.exists() {
                    target_dir = exe_dir_str;
                    is_portable = true;
                // インストール設定が存在せず、ポータブル設定もない (新規) -> ポータブル (exe同階層に作成)
                } else if !installed_config.exists() {
                    target_dir = exe_dir_str;
                    is_portable = true;
                }
                // それ以外 -> インストール使用
            }
        }

        if is_portable {
            log::info!(
                "Portable/Local mode active. Using config from: {}",
                target_dir
            );
        } else {
            log::info!(
                "Installed/Roaming mode active. Using config from: {}",
                target_dir
            );
        }

        let config_path = format!("{}/config.json", target_dir);
        let commands_path = format!("{}/commands.json", target_dir);

        Self {
            config_path,
            commands_path,
        }
    }

    /// Configを読み込む
    pub fn load_config(&self) -> Result<Config, String> {
        // config.jsonが存在しない場合、デフォルト値を保存して返す
        if !Path::new(&self.config_path).exists() {
            let default_config = Config::default();
            self.save_config(&default_config)?;
            return Ok(default_config);
        }

        // config.jsonを読み込む
        let content = fs::read_to_string(&self.config_path).map_err(|e| {
            format!(
                "Failed to read app config file '{}': {}",
                self.config_path, e
            )
        })?;

        // config.jsonが空の場合、デフォルト値を保存して返す
        if content.trim().is_empty() {
            let default_config = Config::default();
            self.save_config(&default_config)?;
            return Ok(default_config);
        }

        // config.jsonをパースして返す
        let mut config: Config = Config::from_json_with_validation(&content).map_err(|e| {
            format!(
                "App config file '{}' validation failed: {}",
                self.config_path, e
            )
        })?;

        // fuzzy_thresholdを検証して必要なら修正
        self.validate_and_fix_fuzzy_threshold(&mut config)?;

        Ok(config)
    }

    /// Commandsを読み込む
    pub fn load_commands(&self) -> Result<Commands, String> {
        // commands.jsonが存在しない場合、デフォルト値を保存して返す
        if !Path::new(&self.commands_path).exists() {
            let default_commands: Commands = Vec::new();
            self.save_commands(&default_commands)?;
            return Ok(default_commands);
        }

        // commands.jsonを読み込む
        let content = fs::read_to_string(&self.commands_path).map_err(|e| {
            format!(
                "Failed to read commands file '{}': {}",
                self.commands_path, e
            )
        })?;

        // commands.jsonが空の場合、デフォルト値を保存して返す
        if content.trim().is_empty() {
            let default_commands: Commands = Vec::new();
            self.save_commands(&default_commands)?;
            return Ok(default_commands);
        }

        // commands.jsonを検証してパース
        // Note: models.rs で定義した関数を使用
        let commands =
            crate::models::commands_from_json_with_validation(&content).map_err(|e| {
                format!(
                    "Commands file '{}' validation failed: {}",
                    self.commands_path, e
                )
            })?;

        Ok(commands)
    }

    /// 設定を保存
    pub fn save_config(&self, config: &Config) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize app config: {}", e))?;

        self.write_file_atomically(&self.config_path, &content)
    }

    /// コマンドを保存
    pub fn save_commands(&self, commands: &Commands) -> Result<(), String> {
        let content = serde_json::to_string_pretty(commands)
            .map_err(|e| format!("Failed to serialize commands: {}", e))?;

        self.write_file_atomically(&self.commands_path, &content)
    }

    /// ファイルを原子的に書き込み
    fn write_file_atomically(&self, file_path: &str, content: &str) -> Result<(), String> {
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = Path::new(file_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
        }

        // 一時ファイルに書き込んでから移動（原子的操作）
        let temp_path = format!("{}.tmp", file_path);
        fs::write(&temp_path, content)
            .map_err(|e| format!("Failed to write temporary file '{}': {}", temp_path, e))?;

        // 一時ファイルを元のファイルに移動
        fs::rename(&temp_path, file_path).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            format!("Failed to save file '{}': {}", file_path, e)
        })?;

        Ok(())
    }

    /// 設定ファイルのパスを取得
    pub fn get_config_path(&self) -> &str {
        &self.config_path
    }

    /// コマンドファイルのパスを取得
    pub fn get_commands_path(&self) -> &str {
        &self.commands_path
    }

    /// 設定ファイルのfuzzy_thresholdの検証と修正
    fn validate_and_fix_fuzzy_threshold(
        &self,
        config: &mut crate::models::Config,
    ) -> Result<(), String> {
        // 0 - 1 の範囲でなければ、0.5 に設定
        if config.fuzzy_threshold < 0.0 || config.fuzzy_threshold > 1.0 {
            log::warn!(
                "Invalid fuzzy_threshold: {}, using default 0.5",
                config.fuzzy_threshold
            );
            config.fuzzy_threshold = 0.5;
        }
        Ok(())
    }
}
