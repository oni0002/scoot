use crate::models::Command;
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

/// ブックマークを読み込む
pub async fn load_bookmarks(
    config: &crate::models::BookmarkConfig,
) -> Result<Vec<Command>, String> {
    let bookmark_path = get_bookmark_path(&config.browser)?;

    if !bookmark_path.exists() {
        return Err(format!("Bookmark file not found: {:?}", bookmark_path));
    }

    let content = fs::read_to_string(&bookmark_path)
        .await
        .map_err(|e| format!("Failed to read bookmark file: {}", e))?;

    let root: ChromiumBookmarkRoot = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse bookmark file: {}", e))?;

    let mut commands = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();

    // ブックマークバーから読み込み
    extract_bookmarks(
        &root.roots.bookmark_bar,
        &mut commands,
        &mut seen_urls,
        &config.prompt,
    );

    // その他のブックマークから読み込み
    extract_bookmarks(
        &root.roots.other,
        &mut commands,
        &mut seen_urls,
        &config.prompt,
    );

    // 同期されたブックマークから読み込み（存在する場合）
    if let Some(synced) = &root.roots.synced {
        extract_bookmarks(synced, &mut commands, &mut seen_urls, &config.prompt);
    }

    Ok(commands)
}

/// ブックマークファイルのパスを取得
fn get_bookmark_path(browser: &str) -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Could not find home directory".to_string())?;
    let bookmark_path = match browser {
        "brave" => {
            home_dir.join("AppData/Local/BraveSoftware/Brave-Browser/User Data/Default/Bookmarks")
        }
        "chrome" => home_dir.join("AppData/Local/Google/Chrome/User Data/Default/Bookmarks"),
        "edge" => home_dir.join("AppData/Local/Microsoft/Edge/User Data/Default/Bookmarks"),
        _ => {
            return Err(format!("Unsupported browser: {}", browser));
        }
    };
    Ok(bookmark_path)
}

/// ブックマークを抽出
fn extract_bookmarks(
    bookmark: &ChromiumBookmark,
    commands: &mut Vec<Command>,
    seen_urls: &mut std::collections::HashSet<String>,
    prompt: &Option<String>,
) {
    if bookmark.bookmark_type == "url" {
        if let Some(url) = &bookmark.url {
            // Check if URL has already been added
            if seen_urls.contains(url) {
                return;
            }

            let command = Command {
                id: format!("bookmark-{}", uuid::Uuid::new_v4()),
                name: bookmark.name.clone(),
                category: "bookmark".to_string(),
                command: url.clone(),
                description: format!("Bookmark: {}", url),
                prompt: prompt.clone(),
                working_dir: None,
                is_editable: false,
            };
            commands.push(command);
            seen_urls.insert(url.clone());
        }
    } else if bookmark.bookmark_type == "folder" {
        if let Some(children) = &bookmark.children {
            for child in children {
                extract_bookmarks(child, commands, seen_urls, prompt);
            }
        }
    }
}
