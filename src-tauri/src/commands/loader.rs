use crate::commands::domain::Command;
use crate::commands::registry::CommandRegistry;
use crate::error::AppError;
use std::collections::HashSet;
use tauri::Emitter;

/// Reload commands, bookmarks, and apps into the registry
pub async fn reload(
    app_handle: &tauri::AppHandle,
    command_store: &crate::commands::store::CommandStore,
    command_registry: &std::sync::Mutex<CommandRegistry>,
    config: &crate::config::domain::Config,
) -> Result<(), AppError> {
    log::debug!("Loading commands, bookmarks, applications, and markdown links in parallel");

    let (commands_result, bookmarks, app_commands, markdown_commands) = tokio::join!(
        command_store.load(),
        async {
            if config.bookmarks.enabled {
                crate::commands::bookmark::load(&config.bookmarks)
                    .await
                    .unwrap_or_else(|e| {
                        log::warn!("Failed to load bookmarks: {}", e);
                        Vec::new()
                    })
            } else {
                Vec::new()
            }
        },
        async {
            if config.applications.enabled {
                crate::commands::application::load(
                    &config.applications.directories,
                    &config.applications.extensions,
                )
                .await
                .unwrap_or_else(|e| {
                    log::warn!("Failed to load applications: {}", e);
                    Vec::new()
                })
            } else {
                Vec::new()
            }
        },
        async {
            if config.markdown.enabled {
                crate::commands::markdown::load(&config.markdown)
                    .await
                    .unwrap_or_else(|e| {
                        log::warn!("Failed to load markdown links: {}", e);
                        Vec::new()
                    })
            } else {
                Vec::new()
            }
        },
    );

    let mut commands = match commands_result {
        Ok(cmds) => cmds,
        Err(e) => {
            let msg = format!("Failed to load commands.json: {}", e);
            log::error!("{}", msg);
            if let Err(emit_err) = app_handle.emit("config-load-error", &msg) {
                log::error!("Failed to emit config-load-error: {}", emit_err);
            }
            Vec::new()
        }
    };
    for cmd in &mut commands {
        cmd.source = crate::commands::domain::SOURCE_USER.to_string();
    }

    let ignored: HashSet<&str> = config.ignored.iter().map(String::as_str).collect();
    let external: Vec<Command> = bookmarks
        .into_iter()
        .chain(app_commands)
        .chain(markdown_commands)
        .chain(crate::commands::domain::get_scoot_commands())
        .filter(|c| !ignored.contains(c.command.as_str()))
        .collect();

    let mut manager = command_registry.lock().map_err(AppError::lock)?;
    manager.set_user_commands(commands);
    manager.set_external_commands(external);

    Ok(())
}
