use crate::error::AppError;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Parse JSON text leniently (json5: comments and trailing commas are allowed in hand-edited files)
pub fn parse<T: DeserializeOwned>(json_str: &str) -> Result<T, crate::error::AppError> {
    json5::from_str(json_str)
        .map_err(|e| crate::error::AppError::Validation(format!("Failed to parse JSON: {}", e)))
}

/// What a JSON file on disk holds
pub enum FileText {
    Missing,
    /// Present but empty or whitespace only. An editor that truncates before it writes leaves the
    /// file like this for a moment, so it is not a reason to overwrite it.
    Blank,
    Text(String),
}

pub async fn read_text(path: &str) -> Result<FileText, AppError> {
    if !tokio::fs::try_exists(path).await.unwrap_or(false) {
        return Ok(FileText::Missing);
    }
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| AppError::System(format!("Failed to read '{}': {}", path, e)))?;
    Ok(if content.trim().is_empty() {
        FileText::Blank
    } else {
        FileText::Text(content)
    })
}

/// Write through a temp file in the same directory and rename it over `path`, so neither the
/// file watcher nor a reader ever sees a half-written file.
pub async fn write_atomic(path: &str, content: impl AsRef<[u8]>) -> Result<(), AppError> {
    // Two writes of one file would share the temp file; the files are small and rarely written
    static WRITING: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    let _writing = WRITING.lock().await;
    let temp = format!("{}.tmp", path);
    let write = async {
        tokio::fs::write(&temp, content).await?;
        tokio::fs::rename(&temp, path).await
    };
    write.await.map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        AppError::System(format!("Failed to write to file: {}", e))
    })
}

/// Serialize as pretty JSON and write it atomically
pub async fn write_pretty<T: Serialize>(path: &str, value: &T) -> Result<(), AppError> {
    let content = serde_json::to_string_pretty(value)
        .map_err(|e| AppError::System(format!("Failed to serialize data: {}", e)))?;
    write_atomic(path, content).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    fn temp_path(name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("scoot-json-{}", nanoid::nanoid!()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name).to_string_lossy().into_owned()
    }

    #[tokio::test]
    async fn atomic_write_replaces_the_file_and_leaves_no_temp() {
        let path = temp_path("a.json");
        write_atomic(&path, "one").await.unwrap();
        write_atomic(&path, "two").await.unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "two");
        assert!(!std::path::Path::new(&format!("{}.tmp", path)).exists());
    }

    #[tokio::test]
    async fn read_text_tells_missing_blank_and_text_apart() {
        let path = temp_path("b.json");
        assert!(matches!(read_text(&path).await.unwrap(), FileText::Missing));
        std::fs::write(&path, " \n").unwrap();
        assert!(matches!(read_text(&path).await.unwrap(), FileText::Blank));
        std::fs::write(&path, "{}").unwrap();
        assert!(matches!(read_text(&path).await.unwrap(), FileText::Text(t) if t == "{}"));
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Item {
        name: String,
        value: u32,
    }

    #[test]
    fn parses_valid_json() {
        let item: Item = parse(r#"{"name": "hello", "value": 42}"#).unwrap();
        assert_eq!(
            item,
            Item {
                name: "hello".to_string(),
                value: 42
            }
        );
    }

    #[test]
    fn accepts_comments_and_trailing_commas() {
        let item: Item = parse(
            "{ // note
 name: 'a', value: 1, }",
        )
        .unwrap();
        assert_eq!(item.value, 1);
    }

    #[test]
    fn reports_syntax_errors_as_validation_errors() {
        let msg = parse::<Item>(r#"{"name": "hello", "value":}"#)
            .unwrap_err()
            .to_string();
        assert!(msg.contains("Validation error"), "got: {}", msg);
    }

    #[test]
    fn rejects_missing_fields_and_wrong_types() {
        assert!(parse::<Item>(r#"{"name": "hello"}"#).is_err());
        assert!(parse::<Item>(r#"{"name": "hello", "value": "x"}"#).is_err());
        assert!(parse::<Item>("").is_err());
    }
}
