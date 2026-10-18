use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub const ERROR: &str = "error";
pub const WARNING: &str = "warning";

/// A problem the user should hear about, found by something that runs without them: startup, a
/// file change, the timer. The window is usually hidden then, so a toast raised on the spot would
/// be gone before anyone looked.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Notice {
    /// What raised it. A source holds one notice at a time, until the cause is gone.
    pub source: &'static str,
    pub level: &'static str,
    pub message: String,
}

/// The notices that still hold, by source. The page asks for them when the window is shown.
#[derive(Default)]
pub struct NoticeState {
    by_source: Mutex<BTreeMap<&'static str, Notice>>,
}

impl NoticeState {
    pub fn raise(&self, notice: Notice) {
        if let Ok(mut notices) = self.by_source.lock() {
            notices.insert(notice.source, notice);
        }
    }

    pub fn clear(&self, source: &str) {
        if let Ok(mut notices) = self.by_source.lock() {
            notices.remove(source);
        }
    }

    pub fn all(&self) -> Vec<Notice> {
        self.by_source
            .lock()
            .map(|notices| notices.values().cloned().collect())
            .unwrap_or_default()
    }
}

/// Keep the notice. If the window is showing, also send it to the page at once; if not, the page
/// asks for it when the window is shown (`get_notices`). The page shows each one once; see
/// `useAppEvents`.
pub fn raise(app_handle: &AppHandle, source: &'static str, level: &'static str, message: String) {
    let notice = Notice {
        source,
        level,
        message,
    };
    if let Some(state) = app_handle.try_state::<NoticeState>() {
        state.raise(notice.clone());
    }
    if crate::window::is_main_window_visible(app_handle) {
        let _ = app_handle.emit("notice", notice);
    }
}

/// The cause is gone (a reload worked, a key was taken): stop reminding
pub fn clear(app_handle: &AppHandle, source: &str) {
    if let Some(state) = app_handle.try_state::<NoticeState>() {
        state.clear(source);
    }
}

/// The notices that still hold (Tauri command)
#[tauri::command]
pub fn get_notices(state: tauri::State<'_, NoticeState>) -> Vec<Notice> {
    state.all()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notice(source: &'static str, message: &str) -> Notice {
        Notice {
            source,
            level: ERROR,
            message: message.to_string(),
        }
    }

    #[test]
    fn a_source_holds_one_notice_the_latest() {
        let state = NoticeState::default();
        state.raise(notice("config", "broken"));
        state.raise(notice("config", "still broken"));
        assert_eq!(state.all(), vec![notice("config", "still broken")]);
    }

    #[test]
    fn sources_are_independent_and_cleared_one_at_a_time() {
        let state = NoticeState::default();
        state.raise(notice("config", "a"));
        state.raise(notice("hotkey", "b"));
        state.clear("config");
        assert_eq!(state.all(), vec![notice("hotkey", "b")]);
        state.clear("config"); // nothing to clear is fine
        state.clear("hotkey");
        assert!(state.all().is_empty());
    }

    #[test]
    fn notices_reach_the_page_as_source_level_and_message() {
        let json = serde_json::to_string(&notice("hotkey", "taken")).unwrap();
        assert_eq!(
            json,
            r#"{"source":"hotkey","level":"error","message":"taken"}"#
        );
    }
}
