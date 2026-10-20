//! Code that needs an `AppHandle`, run against Tauri's mock runtime with its files in a fresh
//! folder. These are integration tests because the mock runtime needs the Common Controls
//! manifest, which `build.rs` embeds in test targets only (see `test.manifest`).

use scoot_lib::commands::registry::CommandRegistry;
use scoot_lib::commands::store::CommandStore;
use scoot_lib::notices::{self, NoticeState, ERROR, WARNING};
use scoot_lib::settings::domain::Settings;
use scoot_lib::settings::store::SettingsStore;
use scoot_lib::state::{CommandsState, SettingsState};
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::{Listener, Manager, WebviewUrl, WebviewWindowBuilder};

fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("scoot-mock-app-{}", nanoid::nanoid!()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// An app whose settings.json and commands.json live in `dir`, with the state `reload` needs
fn app_in(dir: &std::path::Path) -> tauri::App<MockRuntime> {
    let app = mock_builder()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .build(mock_context(noop_assets()))
        .expect("mock app");
    app.manage(CommandsState::new(
        CommandRegistry::default(),
        CommandStore::in_dir(dir),
    ));
    app.manage(SettingsState::new(
        Settings::default(),
        SettingsStore::in_dir(dir),
        None,
    ));
    app.manage(NoticeState::default());
    app
}

fn command_names(app: &tauri::App<MockRuntime>) -> Vec<String> {
    let state = app.state::<CommandsState>();
    let commands = state.commands.lock().unwrap().get_all_commands();
    commands.into_iter().map(|c| c.name).collect()
}

fn held_notices(app: &tauri::App<MockRuntime>) -> Vec<notices::Notice> {
    app.state::<NoticeState>().all()
}

const ONE_COMMAND: &str = r#"[{ "name": "Docs", "category": "url", "command": "https://example.com", "description": "Docs" }]"#;

mod reload {
    use super::*;
    use scoot_lib::lifecycle::reload;

    #[tokio::test]
    async fn broken_settings_still_load_the_commands_and_leave_a_notice() {
        let dir = temp_dir();
        std::fs::write(dir.join("settings.json"), r#"{ "maxResults": 0 }"#).unwrap();
        std::fs::write(dir.join("commands.json"), ONE_COMMAND).unwrap();
        let app = app_in(&dir);

        let result = reload(app.handle()).await;

        assert!(result.is_err(), "the settings error is returned at the end");
        assert!(command_names(&app).contains(&"Docs".to_string()));
        let held = held_notices(&app);
        assert_eq!(held.len(), 1);
        assert_eq!(held[0].source, "settings");
        assert_eq!(held[0].level, ERROR);
    }

    #[tokio::test]
    async fn the_last_good_settings_stay_in_force_while_the_file_is_broken() {
        let dir = temp_dir();
        std::fs::write(dir.join("settings.json"), r#"{ "maxResults": 0 }"#).unwrap();
        let app = app_in(&dir);
        let before = app
            .state::<SettingsState>()
            .settings
            .lock()
            .unwrap()
            .clone();

        let _ = reload(app.handle()).await;

        let after = app
            .state::<SettingsState>()
            .settings
            .lock()
            .unwrap()
            .clone();
        assert_eq!(after.max_results, before.max_results);
    }

    #[tokio::test]
    async fn fixing_the_file_clears_the_notice() {
        let dir = temp_dir();
        std::fs::write(dir.join("settings.json"), r#"{ "maxResults": 0 }"#).unwrap();
        let app = app_in(&dir);
        let _ = reload(app.handle()).await;
        assert_eq!(held_notices(&app).len(), 1);

        std::fs::write(dir.join("settings.json"), r#"{ "maxResults": 7 }"#).unwrap();
        reload(app.handle()).await.unwrap();

        assert!(held_notices(&app).is_empty());
        assert_eq!(
            app.state::<SettingsState>()
                .settings
                .lock()
                .unwrap()
                .max_results,
            7
        );
    }

    #[tokio::test]
    async fn a_commands_file_that_stops_parsing_keeps_the_commands_and_refuses_edits() {
        let dir = temp_dir();
        std::fs::write(dir.join("settings.json"), "{}").unwrap();
        std::fs::write(dir.join("commands.json"), ONE_COMMAND).unwrap();
        let app = app_in(&dir);
        reload(app.handle()).await.unwrap();

        std::fs::write(dir.join("commands.json"), "[ not json").unwrap();
        reload(app.handle()).await.unwrap();

        assert!(command_names(&app).contains(&"Docs".to_string()));
        let state = app.state::<CommandsState>();
        assert!(state.commands.lock().unwrap().ensure_writable().is_err());

        std::fs::write(dir.join("commands.json"), ONE_COMMAND).unwrap();
        reload(app.handle()).await.unwrap();
        assert!(state.commands.lock().unwrap().ensure_writable().is_ok());
    }

    #[tokio::test]
    async fn a_settings_file_equal_to_the_settings_in_memory_does_not_rescan() {
        use scoot_lib::lifecycle::reload_if_settings_differ;
        let dir = temp_dir();
        std::fs::write(dir.join("settings.json"), "{}").unwrap();
        let app = app_in(&dir);
        reload(app.handle()).await.unwrap();

        // A command added by hand is only seen by a reload that really runs
        std::fs::write(dir.join("commands.json"), ONE_COMMAND).unwrap();
        reload_if_settings_differ(app.handle()).await.unwrap();
        assert!(!command_names(&app).contains(&"Docs".to_string()));

        std::fs::write(dir.join("settings.json"), r#"{ "maxResults": 7 }"#).unwrap();
        reload_if_settings_differ(app.handle()).await.unwrap();
        assert!(command_names(&app).contains(&"Docs".to_string()));
    }
    #[tokio::test]
    async fn a_commands_file_that_will_not_parse_leaves_its_own_notice() {
        let dir = temp_dir();
        std::fs::write(dir.join("settings.json"), "{}").unwrap();
        std::fs::write(dir.join("commands.json"), "[ not json").unwrap();
        let app = app_in(&dir);

        let _ = reload(app.handle()).await;

        assert!(held_notices(&app).iter().any(|n| n.source == "commands"));
    }
}

mod notices_reach_the_page {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// An app and a count of the "notice" events it emits. The mock runtime reports every window
    /// as visible, so "not visible" can only be tested as "no window yet", which is what startup is.
    fn app_counting_notices(with_window: bool) -> (tauri::App<MockRuntime>, Arc<AtomicUsize>) {
        let app = app_in(&temp_dir());
        if with_window {
            WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
                .build()
                .expect("main window");
        }
        let sent = Arc::new(AtomicUsize::new(0));
        let counter = sent.clone();
        app.listen("notice", move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
        });
        (app, sent)
    }

    #[test]
    fn without_a_visible_window_the_notice_is_kept_but_not_sent() {
        let (app, sent) = app_counting_notices(false);
        notices::raise(app.handle(), "hotkey", WARNING, "taken".to_string());
        assert_eq!(sent.load(Ordering::SeqCst), 0);
        assert_eq!(held_notices(&app).len(), 1);
    }

    #[test]
    fn a_visible_window_is_told_at_once_and_the_notice_waits_too() {
        let (app, sent) = app_counting_notices(true);
        notices::raise(app.handle(), "hotkey", WARNING, "taken".to_string());
        assert_eq!(sent.load(Ordering::SeqCst), 1);
        assert_eq!(held_notices(&app).len(), 1);
    }

    #[test]
    fn clearing_a_source_drops_its_notice() {
        let (app, _) = app_counting_notices(false);
        notices::raise(app.handle(), "hotkey", WARNING, "taken".to_string());
        notices::clear(app.handle(), "hotkey");
        assert!(held_notices(&app).is_empty());
    }
}
