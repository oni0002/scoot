//! Launch at login: a shortcut to this exe in the user's Startup folder. The folder is the source
//! of truth, so nothing is stored in settings.json and the shortcut can also be removed by hand.

use crate::error::AppError;
use std::path::{Path, PathBuf};

const SHORTCUT_NAME: &str = "Scoot.lnk";
const STARTUP_DIR: &str = r"%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup";

fn shortcut_path() -> PathBuf {
    PathBuf::from(crate::os::expand_env_vars(STARTUP_DIR)).join(SHORTCUT_NAME)
}

/// Whether the Startup folder holds the shortcut
#[tauri::command]
pub fn get_startup_shortcut() -> bool {
    shortcut_path().exists()
}

/// Create or remove the shortcut, and return whether it exists afterwards. Moving the exe leaves a
/// stale shortcut behind; creating it again points it at the new place.
#[tauri::command]
pub async fn set_startup_shortcut(enabled: bool) -> Result<bool, AppError> {
    // COM wants its own thread, not one the async runtime may reuse
    tokio::task::spawn_blocking(move || {
        let path = shortcut_path();
        if enabled {
            crate::shell_link::create(&path, &std::env::current_exe()?, &crate::os::app_dir())?;
        } else {
            remove(&path)?;
        }
        Ok(path.exists())
    })
    .await
    .map_err(|e| AppError::System(format!("Startup shortcut task failed: {e}")))?
}

fn remove(path: &Path) -> Result<(), AppError> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
        _ => Ok(()),
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    use crate::shell_link::create;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("scoot-startup-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn creates_a_shortcut_and_removes_it() {
        let dir = temp_dir("create");
        let path = dir.join("nested").join(SHORTCUT_NAME);
        let exe = std::env::current_exe().unwrap();

        create(&path, &exe, exe.parent().unwrap()).unwrap();
        assert!(path.exists());

        remove(&path).unwrap();
        assert!(!path.exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn creating_again_replaces_the_shortcut() {
        let dir = temp_dir("replace");
        let path = dir.join(SHORTCUT_NAME);
        let exe = std::env::current_exe().unwrap();

        create(&path, &exe, exe.parent().unwrap()).unwrap();
        create(&path, &exe, exe.parent().unwrap()).unwrap();
        assert!(path.exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn removing_a_missing_shortcut_is_not_an_error() {
        assert!(remove(&temp_dir("missing").join(SHORTCUT_NAME)).is_ok());
    }

    #[test]
    fn the_shortcut_lives_in_the_startup_folder() {
        let path = shortcut_path();
        assert!(path.ends_with(format!(r"Startup\{SHORTCUT_NAME}")));
    }
}
