use crate::commands::domain::Command;
use crate::commands::registry::CommandRegistry;
use crate::error::AppError;

fn filter_ignored(commands: Vec<Command>, ignored: &[String]) -> Vec<Command> {
    commands
        .into_iter()
        .filter(|c| !ignored.contains(&c.command))
        .collect()
}

/// Reload commands, bookmarks, and apps into the registry
pub async fn reload(
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
            log::error!(
                "Failed to load commands.json: {}. Proceeding with empty commands.",
                e
            );
            Vec::new()
        }
    };
    for cmd in &mut commands {
        cmd.source = crate::commands::domain::SOURCE_USER.to_string();
    }

    let bookmarks = filter_ignored(bookmarks, &config.ignored);
    let app_commands = filter_ignored(app_commands, &config.ignored);
    let markdown_commands = filter_ignored(markdown_commands, &config.ignored);
    let scoot_commands =
        filter_ignored(crate::commands::domain::get_scoot_commands(), &config.ignored);

    let mut external = bookmarks;
    external.extend(app_commands);
    external.extend(markdown_commands);
    external.extend(scoot_commands);

    let mut manager = command_registry
        .lock()
        .map_err(|e| AppError::lock(e))?;
    manager.set_user_commands(commands);
    manager.set_external_commands(external);

    Ok(())
}
