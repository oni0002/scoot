use crate::commands::domain::{Command, CATEGORY_URL, SOURCE_BOOKMARK};
use crate::commands::execution::is_url_scheme_allowed;
use crate::settings::domain::BookmarkSettings;

use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum BookmarkType {
    Url,
    Folder,
}

#[derive(Debug, Deserialize)]
struct ChromiumBookmark {
    name: String,
    url: Option<String>,
    #[serde(rename = "type")]
    bookmark_type: BookmarkType,
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
pub async fn load(settings: &BookmarkSettings) -> Result<Vec<Command>, crate::error::AppError> {
    let bookmark_path = get_bookmark_path(&settings.browser)?;

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
    collect(&root.roots.bookmark_bar, &mut commands, &mut seen_urls);

    // Load bookmarks from the other bookmarks
    collect(&root.roots.other, &mut commands, &mut seen_urls);

    // Load bookmarks from the synced bookmarks (if they exist)
    if let Some(synced) = &root.roots.synced {
        collect(synced, &mut commands, &mut seen_urls);
    }

    Ok(commands)
}

/// Get the path to the bookmark file, under %LOCALAPPDATA% (which need not be below the home
/// directory). Only brave, chrome, and edge are supported.
fn get_bookmark_path(browser: &str) -> Result<PathBuf, crate::error::AppError> {
    let local_app_data = std::env::var_os("LOCALAPPDATA").ok_or_else(|| {
        crate::error::AppError::System("The LOCALAPPDATA variable is not set".to_string())
    })?;
    bookmark_path_in(Path::new(&local_app_data), browser)
}

fn bookmark_path_in(
    local_app_data: &Path,
    browser: &str,
) -> Result<PathBuf, crate::error::AppError> {
    let relative = match browser {
        "brave" => "BraveSoftware/Brave-Browser/User Data/Default/Bookmarks",
        "chrome" => "Google/Chrome/User Data/Default/Bookmarks",
        "edge" => "Microsoft/Edge/User Data/Default/Bookmarks",
        _ => {
            return Err(crate::error::AppError::System(format!(
                "Unsupported browser: {}",
                browser
            )));
        }
    };
    Ok(local_app_data.join(relative))
}

/// Collect commands from bookmarks
fn collect(
    bookmark: &ChromiumBookmark,
    commands: &mut Vec<Command>,
    seen_urls: &mut std::collections::HashSet<String>,
) {
    if bookmark.bookmark_type == BookmarkType::Url {
        if let Some(url) = &bookmark.url {
            // Bookmarklets (javascript:) and browser pages (chrome://) cannot be opened from here
            if !is_url_scheme_allowed(url) {
                return;
            }
            // Skip duplicate URLs
            if seen_urls.contains(url) {
                return;
            }

            commands.push(Command::new(
                bookmark.name.clone(),
                CATEGORY_URL,
                SOURCE_BOOKMARK,
                url.clone(),
                url.clone(),
            ));
            seen_urls.insert(url.clone());
        }
    } else if bookmark.bookmark_type == BookmarkType::Folder {
        if let Some(children) = &bookmark.children {
            for child in children {
                collect(child, commands, seen_urls);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bookmark_files_live_under_local_app_data_wherever_that_is() {
        // %LOCALAPPDATA% can be moved off the home directory, so nothing may assume where it is
        let base = Path::new("D:/Redirected/Local");
        for (browser, tail) in [
            (
                "brave",
                "BraveSoftware/Brave-Browser/User Data/Default/Bookmarks",
            ),
            ("chrome", "Google/Chrome/User Data/Default/Bookmarks"),
            ("edge", "Microsoft/Edge/User Data/Default/Bookmarks"),
        ] {
            assert_eq!(bookmark_path_in(base, browser).unwrap(), base.join(tail));
        }
    }

    #[test]
    fn only_links_that_can_be_opened_are_collected() {
        let folder: ChromiumBookmark = serde_json::from_str(
            r#"{
                "name": "bar", "type": "folder",
                "children": [
                    {"name": "Docs", "type": "url", "url": "https://example.com/docs"},
                    {"name": "Docs again", "type": "url", "url": "https://example.com/docs"},
                    {"name": "Mail", "type": "url", "url": "mailto:someone@example.com"},
                    {"name": "Bookmarklet", "type": "url", "url": "javascript:alert(1)"},
                    {"name": "Settings", "type": "url", "url": "chrome://settings"},
                    {"name": "Nested", "type": "folder", "children": [
                        {"name": "Files", "type": "url", "url": "ftp://files.example.com"}
                    ]}
                ]
            }"#,
        )
        .unwrap();

        let mut commands = Vec::new();
        collect(
            &folder,
            &mut commands,
            &mut std::collections::HashSet::new(),
        );

        let urls: Vec<&str> = commands.iter().map(|c| c.command.as_str()).collect();
        assert_eq!(
            urls,
            [
                "https://example.com/docs",
                "mailto:someone@example.com",
                "ftp://files.example.com"
            ]
        );
    }

    #[test]
    fn an_unsupported_browser_is_an_error() {
        let err = bookmark_path_in(Path::new("C:/Local"), "firefox").unwrap_err();
        assert!(err.to_string().contains("Unsupported browser: firefox"));
    }
}
