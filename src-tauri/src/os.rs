use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;

static ENV_VAR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"%([^%]+)%").expect("invalid regex"));
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

/// Open the specified path (file, directory, URL) with the default application
pub fn open_path(app_handle: &AppHandle, path: &str) -> Result<(), crate::error::AppError> {
    app_handle
        .opener()
        .open_path(path, None::<&str>)
        .map_err(|e| {
            crate::error::AppError::System(format!("Failed to open path '{}': {}", path, e))
        })
}

/// Open the specified URL with the default browser
pub fn open_url(app_handle: &AppHandle, url: &str) -> Result<(), crate::error::AppError> {
    app_handle
        .opener()
        .open_url(url, None::<&str>)
        .map_err(|e| crate::error::AppError::System(format!("Failed to open URL '{}': {}", url, e)))
}

/// Resolve the path to a resource file
pub fn resolve_resource(
    app_handle: &AppHandle,
    path: &str,
) -> Result<PathBuf, crate::error::AppError> {
    let resource_path = app_handle
        .path()
        .resolve(path, tauri::path::BaseDirectory::Resource)
        .map_err(|e| {
            crate::error::AppError::System(format!(
                "Failed to resolve resource path '{}': {}",
                path, e
            ))
        })?;

    if !resource_path.exists() {
        return Err(crate::error::AppError::NotFound(format!(
            "Resource not found: {}",
            path
        )));
    }

    Ok(resource_path)
}

/// Get the path to the log directory
pub fn get_log_dir(_app_handle: &AppHandle) -> Result<PathBuf, crate::error::AppError> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("logs")))
        .ok_or_else(|| {
            crate::error::AppError::System("Failed to determine log directory".to_string())
        })
}

/// Expand Windows environment variables like %APPDATA%
pub fn expand_env_vars(path: &str) -> String {
    ENV_VAR_RE.replace_all(path, |caps: &regex::Captures| {
        let key = &caps[1];
        std::env::var(key).unwrap_or_else(|_| format!("%{}%", key))
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_expand_existing_var() {
        let key = "SCOOT_TEST_VAR";
        let value = "test_value";
        env::set_var(key, value);

        let input = format!("path/to/%{}%/file", key);
        let expected = format!("path/to/{}/file", value);

        assert_eq!(expand_env_vars(&input), expected);

        env::remove_var(key);
    }

    #[test]
    fn test_expand_non_existing_var() {
        let input = "path/to/%NON_EXISTING_VAR%/file";
        assert_eq!(expand_env_vars(input), input);
    }

    #[test]
    fn test_no_vars() {
        let input = "path/to/file";
        assert_eq!(expand_env_vars(input), input);
    }
}
