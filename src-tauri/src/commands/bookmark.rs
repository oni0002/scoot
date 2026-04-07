use crate::commands::domain::Command;
use crate::config::domain::BookmarkConfig;

use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Deserialize)]
struct ChromiumBookmark {
    name: String,
    url: Option<String>, // Make URL optional since folders don't have URLs
    #[serde(rename = "type")]
    bookmark_type: String,
    children: Option<Vec<ChromiumBookmark>>,
}

#[derive(Debug, Deserialize)]
struct ChromiumBookmarkRoot {
    roots: ChromiumBookmarkRoots,
}

#[derive(Debug, Deserialize)]
struct ChromiumBookmarkRoots {
    bookmark_bar: ChromiumBookmark,
    other: ChromiumBookmark,
    synced: Option<ChromiumBookmark>,
}

/// Load bookmarks
pub async fn load(config: &BookmarkConfig) -> Result<Vec<Command>, crate::error::AppError> {
    let bookmark_path = get_bookmark_path(&config.browser)?;

    // If the bookmark file doesn't exist, return an error
    if !bookmark_path.exists() {
        return Err(crate::error::AppError::System(format!(
            "Bookmark file not found: {:?}",
            bookmark_path
        )));
    }

    // Read the bookmark file
    let content = fs::read_to_string(&bookmark_path).await.map_err(|e| {
        crate::error::AppError::System(format!("Failed to read bookmark file: {}", e))
    })?;

    // JSON parse (execute in a worker thread)
    let root: ChromiumBookmarkRoot =
        tokio::task::spawn_blocking(move || serde_json::from_str(&content))
            .await
            .map_err(|e| {
                crate::error::AppError::System(format!("Failed to spawn blocking task: {}", e))
            })?
            .map_err(|e| {
                crate::error::AppError::System(format!("Failed to parse bookmark file: {}", e))
            })?;

    let mut commands = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();

    // Load bookmarks from the bookmark bar
    collect(
        &root.roots.bookmark_bar,
        &mut commands,
        &mut seen_urls,
        &config.prompt,
    );

    // Load bookmarks from the other bookmarks
    collect(
        &root.roots.other,
        &mut commands,
        &mut seen_urls,
        &config.prompt,
    );

    // Load bookmarks from the synced bookmarks (if they exist)
    if let Some(synced) = &root.roots.synced {
        collect(synced, &mut commands, &mut seen_urls, &config.prompt);
    }

    Ok(commands)
}

/// Get the path to the bookmark file
/// Only brave, chrome, and edge are supported
fn get_bookmark_path(browser: &str) -> Result<PathBuf, crate::error::AppError> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        crate::error::AppError::System("Could not find home directory".to_string())
    })?;
    let bookmark_path = match browser {
        "brave" => {
            home_dir.join("AppData/Local/BraveSoftware/Brave-Browser/User Data/Default/Bookmarks")
        }
        "chrome" => home_dir.join("AppData/Local/Google/Chrome/User Data/Default/Bookmarks"),
        "edge" => home_dir.join("AppData/Local/Microsoft/Edge/User Data/Default/Bookmarks"),
        _ => {
            return Err(crate::error::AppError::System(format!(
                "Unsupported browser: {}",
                browser
            )));
        }
    };
    Ok(bookmark_path)
}

/// Collect commands from bookmarks
fn collect(
    bookmark: &ChromiumBookmark,
    commands: &mut Vec<Command>,
    seen_urls: &mut std::collections::HashSet<String>,
    prompt: &Option<String>,
) {
    if bookmark.bookmark_type == "url" {
        if let Some(url) = &bookmark.url {
            // Skip duplicate URLs
            if seen_urls.contains(url) {
                return;
            }

            let command = Command {
                id: String::new(),
                name: bookmark.name.clone(),
                category: "url".to_string(),
                source: "bookmark".to_string(),
                command: url.clone(),
                description: url.clone(),
                prompt: prompt.clone(),
                working_dir: None,
                show_window: None,
            };
            commands.push(command);
            seen_urls.insert(url.clone());
        }
    } else if bookmark.bookmark_type == "folder" {
        if let Some(children) = &bookmark.children {
            for child in children {
                collect(child, commands, seen_urls, prompt);
            }
        }
    }
}

/// Start bookmark auto-refresh task
pub fn start_update_task(app_handle: tauri::AppHandle) {
    use tauri::Manager;
    tauri::async_runtime::spawn(async move {
        log::debug!("Starting bookmark auto-refresh task");
        loop {
            // Get refresh interval from current config
            let interval_minutes =
                if let Some(state) = app_handle.try_state::<crate::state::AppState>() {
                    if let Ok(config) = state.config.lock() {
                        // Minimum value limit (1 minute)
                        std::cmp::max(config.bookmarks.refresh_interval_minutes, 1)
                    } else {
                        30 // Lock acquisition failure
                    }
                } else {
                    30 // State acquisition failure
                };

            // Wait for specified time
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_minutes * 60)).await;

            log::debug!("Executing scheduled bookmark refresh...");

            // Get config and reload
            let config_opt = if let Some(state) = app_handle.try_state::<crate::state::AppState>() {
                state.config.lock().ok().map(|c| c.clone())
            } else {
                None
            };

            if let Some(config) = config_opt {
                // Reload bookmarks only
                if let Some(state) = app_handle.try_state::<crate::state::AppState>() {
                    if let Err(e) =
                        crate::commands::ipc::reload_bookmarks(&state.commands, &config).await
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
