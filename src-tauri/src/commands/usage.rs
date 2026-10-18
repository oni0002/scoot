use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// How often and how recently one command was run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub count: u32,
    /// Unix time in milliseconds
    pub last_used: u64,
}

/// Run history, kept in usage.json next to the exe. It only reorders search results, so a missing
/// or unreadable file just means starting over with an empty history.
pub struct UsageTracker {
    path: String,
    entries: Mutex<HashMap<String, Usage>>,
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl UsageTracker {
    pub fn new() -> Self {
        let path = crate::os::app_dir()
            .join("usage.json")
            .to_string_lossy()
            .into_owned();
        let entries = std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default();
        Self {
            path,
            entries: Mutex::new(entries),
        }
    }

    pub fn all(&self) -> Result<HashMap<String, Usage>, crate::error::AppError> {
        Ok(self
            .entries
            .lock()
            .map_err(crate::error::AppError::lock)?
            .clone())
    }

    /// Count one run of `id` and persist the history. A failed write is logged, not surfaced:
    /// the command has already run.
    pub async fn record(&self, id: &str) {
        let json = match self.update(id, now_millis()) {
            Ok(json) => json,
            Err(e) => {
                log::warn!("Failed to record usage: {}", e);
                return;
            }
        };
        if let Err(e) = crate::json::write_atomic(&self.path, json).await {
            log::warn!("Failed to write '{}': {}", self.path, e);
        }
    }

    /// Forget every run, in memory and on disk
    pub async fn clear(&self) -> Result<(), crate::error::AppError> {
        self.entries
            .lock()
            .map_err(crate::error::AppError::lock)?
            .clear();
        crate::json::write_atomic(&self.path, "{}").await
    }

    fn update(&self, id: &str, now: u64) -> Result<String, crate::error::AppError> {
        let mut entries = self.entries.lock().map_err(crate::error::AppError::lock)?;
        let usage = entries.entry(id.to_string()).or_insert(Usage {
            count: 0,
            last_used: now,
        });
        usage.count = usage.count.saturating_add(1);
        usage.last_used = now;
        serde_json::to_string(&*entries).map_err(|e| {
            crate::error::AppError::System(format!("Failed to serialize usage: {}", e))
        })
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Run history (Tauri command)
#[tauri::command]
pub fn get_usage(
    state: tauri::State<'_, crate::state::CommandsState>,
) -> Result<HashMap<String, Usage>, crate::error::AppError> {
    state.usage.all()
}

/// Delete the run history (Tauri command)
#[tauri::command]
pub async fn clear_usage(
    state: tauri::State<'_, crate::state::CommandsState>,
) -> Result<(), crate::error::AppError> {
    state.usage.clear().await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tracker() -> UsageTracker {
        UsageTracker {
            path: String::new(),
            entries: Mutex::new(HashMap::new()),
        }
    }

    #[test]
    fn counts_runs_and_remembers_the_latest() {
        let t = tracker();
        t.update("a", 100).unwrap();
        t.update("a", 200).unwrap();
        t.update("b", 300).unwrap();
        let all = t.all().unwrap();
        assert_eq!(
            all["a"],
            Usage {
                count: 2,
                last_used: 200
            }
        );
        assert_eq!(all["b"].count, 1);
    }

    #[tokio::test]
    async fn clearing_forgets_every_run() {
        let dir = std::env::temp_dir().join(format!("scoot-usage-{}", nanoid::nanoid!()));
        std::fs::create_dir_all(&dir).unwrap();
        let t = UsageTracker {
            path: dir.join("usage.json").to_string_lossy().into_owned(),
            entries: Mutex::new(HashMap::new()),
        };
        t.update("a", 1).unwrap();
        t.clear().await.unwrap();
        assert!(t.all().unwrap().is_empty());
        assert_eq!(std::fs::read_to_string(&t.path).unwrap(), "{}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn serializes_for_the_webview_in_camel_case() {
        let t = tracker();
        let json = t.update("a", 5).unwrap();
        assert_eq!(json, r#"{"a":{"count":1,"lastUsed":5}}"#);
    }
}
