use crate::commands::domain::Command;
use crate::config::domain::MarkdownConfig;
use regex::Regex;
use std::collections::HashSet;

/// Load commands from markdown files
pub async fn load(config: &MarkdownConfig) -> Result<Vec<Command>, crate::error::AppError> {
    let paths = config.paths.clone();

    let commands = tokio::task::spawn_blocking(move || {
        let mut commands = Vec::new();
        let mut seen_urls = HashSet::new();

        // Regex: captures optional '!' prefix + [text](url)
        let link_re = Regex::new(r"(!?)\[([^\]]+)\]\(([^)]+)\)").unwrap();

        for path in &paths {
            let expanded = crate::system::expand_env_vars(path);
            let content = match std::fs::read_to_string(&expanded) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Failed to read markdown file '{}': {}", expanded, e);
                    continue;
                }
            };

            // Extract filename for description
            let file_name = std::path::Path::new(&expanded)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| expanded.clone());

            for cap in link_re.captures_iter(&content) {
                // Skip image links (![text](url))
                if &cap[1] == "!" {
                    continue;
                }

                let name = cap[2].to_string();
                let url = cap[3].trim().to_string();

                // Skip anchor links
                if url.starts_with('#') {
                    continue;
                }

                // Skip duplicates
                if seen_urls.contains(&url) {
                    continue;
                }
                seen_urls.insert(url.clone());

                // Determine category based on URL content
                let category = if url.starts_with("http://")
                    || url.starts_with("https://")
                    || url.starts_with("ftp://")
                    || url.starts_with("mailto:")
                {
                    "url"
                } else {
                    "file"
                };

                commands.push(Command::new(name, category, file_name.clone(), url.clone(), url.clone()));
            }
        }

        commands
    })
    .await
    .map_err(|e| {
        crate::error::AppError::System(format!("Failed to load markdown links: {}", e))
    })?;

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_links() {
        let content = r#"
# My Links

- [Google](https://google.com)
- [GitHub](https://github.com)
- [Local File](C:\path\to\file.txt)
- ![Image](image.png)
- [Anchor](#section)
- [Duplicate](https://google.com)
"#;

        let link_re = Regex::new(r"(!?)\[([^\]]+)\]\(([^)]+)\)").unwrap();
        let mut results = Vec::new();
        let mut seen = HashSet::new();

        for cap in link_re.captures_iter(content) {
            if &cap[1] == "!" {
                continue;
            }
            let name = cap[2].to_string();
            let url = cap[3].trim().to_string();
            if url.starts_with('#') {
                continue;
            }
            if seen.contains(&url) {
                continue;
            }
            seen.insert(url.clone());
            results.push((name, url));
        }

        assert_eq!(results.len(), 3);
        assert_eq!(results[0], ("Google".to_string(), "https://google.com".to_string()));
        assert_eq!(results[1], ("GitHub".to_string(), "https://github.com".to_string()));
        assert_eq!(results[2], ("Local File".to_string(), r"C:\path\to\file.txt".to_string()));
    }
}
