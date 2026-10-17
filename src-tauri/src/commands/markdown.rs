use crate::commands::domain::{Command, CATEGORY_FILE, CATEGORY_URL};
use crate::commands::execution::is_url_scheme_allowed;
use crate::config::domain::MarkdownConfig;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(!?)\[([^\]]+)\]\(([^)]+)\)").expect("invalid regex"));

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

            for (name, url) in parse_links(&content) {
                if !seen_urls.insert(url.clone()) {
                    continue;
                }
                let category = if is_url_scheme_allowed(&url) {
                    CATEGORY_URL
                } else {
                    CATEGORY_FILE
                };
                commands.push(Command::new(
                    name,
                    category,
                    file_name.clone(),
                    url.clone(),
                    url,
                ));
            }
        }

        commands
    })
    .await
    .map_err(|e| crate::error::AppError::System(format!("Failed to load markdown links: {}", e)))?;

    Ok(commands)
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
- [Local File](C:\path	oile.txt)
- ![Image](image.png)
- [Anchor](#section)
"#;

        assert_eq!(
            parse_links(content),
            vec![
                ("Google".to_string(), "https://google.com".to_string()),
                ("GitHub".to_string(), "https://github.com".to_string()),
                ("Local File".to_string(), r"C:\path	oile.txt".to_string()),
            ]
        );
    }
}
