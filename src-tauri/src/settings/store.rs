use crate::error::AppError;
use crate::json::FileText;
use crate::settings::domain::Settings;

// --- Infrastructure / Core Logic ---

const SETTINGS_FILE: &str = "settings.json";
/// The name before 0.3.0. Still read, and renamed to `SETTINGS_FILE`, until 0.4.0 (ADR 0005).
const LEGACY_FILE: &str = "config.json";

pub struct SettingsStore {
    settings_path: String,
    legacy_path: String,
}

/// Settings store
/// Load, save, and validate settings.json
impl SettingsStore {
    pub(crate) fn new() -> Self {
        Self::in_dir(&crate::os::app_dir())
    }

    fn in_dir(dir: &std::path::Path) -> Self {
        let path = |name: &str| dir.join(name).to_string_lossy().into_owned();
        Self {
            settings_path: path(SETTINGS_FILE),
            legacy_path: path(LEGACY_FILE),
        }
    }

    /// Load the settings. A missing file is created with the defaults; a blank one just means the
    /// defaults, and is left for the user to finish writing.
    pub async fn load(&self) -> Result<Settings, crate::error::AppError> {
        let path = self.settle_file().await;
        match crate::json::read_text(&path).await? {
            FileText::Missing => {
                let default_settings = Settings::default();
                self.save(&default_settings).await?;
                Ok(default_settings)
            }
            FileText::Blank => Ok(Settings::default()),
            FileText::Text(content) => Settings::from_json_with_validation(&content).map_err(|e| {
                crate::error::AppError::System(format!(
                    "App settings file '{}' validation failed: {}",
                    path, e
                ))
            }),
        }
    }

    /// The file to read. settings.json wins; failing that, config.json from before the rename is
    /// moved to settings.json, or read where it is if it cannot be moved (ADR 0005).
    async fn settle_file(&self) -> String {
        let exists = |path: &str| tokio::fs::try_exists(path.to_string());
        if exists(&self.settings_path).await.unwrap_or(false) {
            if exists(&self.legacy_path).await.unwrap_or(false) {
                log::warn!(
                    "Both {} and {} exist; using {}",
                    SETTINGS_FILE,
                    LEGACY_FILE,
                    SETTINGS_FILE
                );
            }
            return self.settings_path.clone();
        }
        if !exists(&self.legacy_path).await.unwrap_or(false) {
            return self.settings_path.clone();
        }
        match tokio::fs::rename(&self.legacy_path, &self.settings_path).await {
            Ok(()) => {
                log::info!("Renamed {} to {}", LEGACY_FILE, SETTINGS_FILE);
                self.settings_path.clone()
            }
            Err(e) => {
                log::warn!(
                    "Could not rename {} to {} ({}); reading {} as it is",
                    LEGACY_FILE,
                    SETTINGS_FILE,
                    e,
                    LEGACY_FILE
                );
                self.legacy_path.clone()
            }
        }
    }

    /// Save the settings
    pub async fn save(
        &self,
        settings: &crate::settings::domain::Settings,
    ) -> Result<(), crate::error::AppError> {
        crate::json::write_pretty(&self.settings_path, settings).await
    }

    /// The files a change must be picked up from: settings.json, and config.json while it may
    /// still be the one in use
    pub fn watched_paths(&self) -> [&str; 2] {
        [&self.settings_path, &self.legacy_path]
    }

    /// The file in use, for "Open settings.json"
    pub fn file_in_use(&self) -> &str {
        if !std::path::Path::new(&self.settings_path).exists()
            && std::path::Path::new(&self.legacy_path).exists()
        {
            &self.legacy_path
        } else {
            &self.settings_path
        }
    }
}

/// Reload settings
pub async fn reload(
    settings_store: &SettingsStore,
    state_settings: &std::sync::Mutex<Settings>,
) -> Result<Settings, crate::error::AppError> {
    // Load settings
    log::debug!("Loading configuration.");
    let new_settings = settings_store.load().await?;

    // Update state settings
    {
        let mut locked_settings = state_settings.lock().map_err(AppError::lock)?;
        *locked_settings = new_settings.clone();
    }

    Ok(new_settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("scoot-config-{}", nanoid::nanoid!()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn hotkey_json(hotkey: &str) -> String {
        format!(r#"{{ "hotkey": "{hotkey}" }}"#)
    }

    #[tokio::test]
    async fn a_fresh_start_creates_settings_json_with_the_defaults() {
        let dir = temp_dir();
        let settings = SettingsStore::in_dir(&dir).load().await.unwrap();
        assert_eq!(settings.hotkey, Settings::default().hotkey);
        assert!(dir.join("settings.json").exists());
        assert!(!dir.join("config.json").exists());
    }

    #[tokio::test]
    async fn config_json_is_renamed_on_the_first_load_and_its_values_kept() {
        let dir = temp_dir();
        std::fs::write(dir.join("config.json"), hotkey_json("Ctrl+Alt+K")).unwrap();
        let settings = SettingsStore::in_dir(&dir).load().await.unwrap();
        assert_eq!(settings.hotkey, "Ctrl+Alt+K");
        assert!(dir.join("settings.json").exists());
        assert!(!dir.join("config.json").exists());
    }

    #[tokio::test]
    async fn settings_json_wins_and_config_json_is_left_alone() {
        let dir = temp_dir();
        std::fs::write(dir.join("settings.json"), hotkey_json("Ctrl+Alt+S")).unwrap();
        std::fs::write(dir.join("config.json"), hotkey_json("Ctrl+Alt+C")).unwrap();
        let store = SettingsStore::in_dir(&dir);
        assert_eq!(store.load().await.unwrap().hotkey, "Ctrl+Alt+S");
        store.save(&Settings::default()).await.unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("config.json")).unwrap(),
            hotkey_json("Ctrl+Alt+C")
        );
    }

    #[tokio::test]
    async fn config_json_is_read_where_it_is_when_it_cannot_be_renamed() {
        let dir = temp_dir();
        std::fs::write(dir.join("config.json"), hotkey_json("Ctrl+Alt+K")).unwrap();
        // The target's folder does not exist, so the rename fails
        let store = SettingsStore {
            settings_path: dir
                .join("nowhere")
                .join("settings.json")
                .to_string_lossy()
                .into_owned(),
            legacy_path: dir.join("config.json").to_string_lossy().into_owned(),
        };
        assert_eq!(store.load().await.unwrap().hotkey, "Ctrl+Alt+K");
        assert!(dir.join("config.json").exists());
    }

    #[test]
    fn open_settings_json_opens_the_file_in_use() {
        let dir = temp_dir();
        let store = SettingsStore::in_dir(&dir);
        assert_eq!(store.file_in_use(), store.settings_path);
        std::fs::write(dir.join("config.json"), "{}").unwrap();
        assert_eq!(store.file_in_use(), store.legacy_path);
        std::fs::write(dir.join("settings.json"), "{}").unwrap();
        assert_eq!(store.file_in_use(), store.settings_path);
    }
}
