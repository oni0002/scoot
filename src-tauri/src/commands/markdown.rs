use crate::commands::domain::{Command, CATEGORY_FILE, CATEGORY_URL};
use crate::commands::execution::is_url_scheme_allowed;
use crate::config::domain::MarkdownConfig;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

static LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(!?)\[([^\]]+)\]\(([^)]+)\)").expect("invalid regex"));

/// `scheme:` at the start of a link. At least two characters, so a drive letter (`C:\`) is not one.
static SCHEME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z][A-Za-z0-9+.\-]+:").expect("invalid regex"));

/// Load commands from markdown files
pub async fn load(config: &MarkdownConfig) -> Result<Vec<Command>, crate::error::AppError> {
    let paths = config.paths.clone();

    let commands = tokio::task::spawn_blocking(move || {
        let mut commands = Vec::new();
        let mut seen_urls = HashSet::new();

        for path in &paths {
            let expanded = crate::os::expand_env_vars(path);
            let content = match std::fs::read_to_string(&expanded) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Failed to read markdown file '{}': {}", expanded, e);
                    continue;
                }
            };

            // The file name becomes the Source, shown as the row's badge
            let file_name = std::path::Path::new(&expanded)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| expanded.clone());

            let directory = Path::new(&expanded).parent().unwrap_or(Path::new(""));

            for (name, target) in parse_links(&content) {
                let Some((category, command)) = resolve_target(&target, directory) else {
                    log::debug!(
                        "Skipping link '{}' in '{}': cannot be opened",
                        target,
                        expanded
                    );
                    continue;
                };
                // Compared after resolving: the same relative path in two folders is two files
                if !seen_urls.insert(command.clone()) {
                    continue;
                }
                commands.push(Command::new(
                    name,
                    category,
                    file_name.clone(),
                    command.clone(),
                    command,
                ));
            }
        }

        commands
    })
    .await
    .map_err(|e| crate::error::AppError::System(format!("Failed to load markdown links: {}", e)))?;

    Ok(commands)
}

/// What a link opens, as (category, command): an allowed URL as it is, a path made absolute
/// against the folder of the Markdown file. None for what Scoot cannot open: other schemes
/// (`javascript:`, `obsidian://`, `file://`) are refused by `open_target`, so they are not offered.
fn resolve_target(target: &str, directory: &Path) -> Option<(&'static str, String)> {
    if is_url_scheme_allowed(target) {
        return Some((CATEGORY_URL, target.to_string()));
    }
    if SCHEME_RE.is_match(target) {
        return None;
    }
    // A link to a heading inside another file ("notes.md#intro") opens the file
    let without_fragment = target.split('#').next().unwrap_or(target);
    let expanded = crate::os::expand_env_vars(without_fragment);
    let path = Path::new(&expanded);
    let full = if path.is_absolute() {
        path.to_path_buf()
    } else {
        directory.join(path)
    };
    Some((CATEGORY_FILE, full.to_string_lossy().into_owned()))
}

/// The (text, target) of every link in a Markdown document, skipping images and in-page anchors
fn parse_links(content: &str) -> Vec<(String, String)> {
    LINK_RE
        .captures_iter(content)
        .filter(|cap| &cap[1] != "!")
        .map(|cap| (cap[2].to_string(), cap[3].trim().to_string()))
        .filter(|(_, url)| !url.starts_with('#'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_links_skips_images_and_anchors() {
        let content = r#"
# My Links

- [Google](https://google.com)
- [GitHub](https://github.com)
- [Local File](C:\path\to\file.txt)
- ![Image](image.png)
- [Anchor](#section)
"#;

        assert_eq!(
            parse_links(content),
            vec![
                ("Google".to_string(), "https://google.com".to_string()),
                ("GitHub".to_string(), "https://github.com".to_string()),
                ("Local File".to_string(), r"C:\path\to\file.txt".to_string()),
            ]
        );
    }

    fn resolve(target: &str) -> Option<(&'static str, String)> {
        resolve_target(target, Path::new("D:/notes"))
    }

    #[test]
    fn urls_the_launcher_can_open_are_kept_as_they_are() {
        for url in [
            "https://example.com/a?b=c",
            "http://example.com",
            "ftp://files.example.com",
            "mailto:me@example.com",
        ] {
            assert_eq!(resolve(url), Some((CATEGORY_URL, url.to_string())));
        }
    }

    #[test]
    fn other_schemes_are_not_offered() {
        for target in [
            "javascript:alert(1)",
            "obsidian://open?vault=notes",
            "file:///C:/Windows/notepad.exe",
            "vscode://file/C:/a.txt",
            "data:text/html,x",
        ] {
            assert_eq!(resolve(target), None, "{}", target);
        }
    }

    #[test]
    fn absolute_paths_stay_and_relative_ones_start_in_the_markdown_folder() {
        assert_eq!(
            resolve(r"C:\Users\me\todo.txt"),
            Some((CATEGORY_FILE, r"C:\Users\me\todo.txt".to_string()))
        );
        assert_eq!(
            resolve(r"\\server\share\a.txt"),
            Some((CATEGORY_FILE, r"\\server\share\a.txt".to_string()))
        );
        assert_eq!(
            resolve("sub/todo.txt"),
            Some((
                CATEGORY_FILE,
                Path::new("D:/notes")
                    .join("sub/todo.txt")
                    .to_string_lossy()
                    .into_owned()
            ))
        );
    }

    #[test]
    fn a_heading_in_another_file_opens_the_file() {
        assert_eq!(
            resolve("other.md#intro"),
            Some((
                CATEGORY_FILE,
                Path::new("D:/notes")
                    .join("other.md")
                    .to_string_lossy()
                    .into_owned()
            ))
        );
    }
}
