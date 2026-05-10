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
    log::debug!("Loading commands");
    let mut commands = match command_store.load().await {
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

    log::debug!("Loading bookmarks");
    let bookmarks = if config.bookmarks.enabled {
        crate::commands::bookmark::load(&config.bookmarks)
            .await
            .unwrap_or_else(|e| {
                log::warn!("Failed to load bookmarks: {}", e);
                Vec::new()
            })
    } else {
        Vec::new()
    };

    log::debug!("Loading applications");
    let app_commands = if config.applications.enabled {
        crate::commands::application::load(
            &config.applications.directories,
            &config.applications.extensions,
        )
        .await
        .unwrap_or_default()
    } else {
        Vec::new()
    };

    let bookmarks = filter_ignored(bookmarks, &config.ignored);
    let app_commands = filter_ignored(app_commands, &config.ignored);

    log::debug!("Loading markdown links");
    let markdown_commands = if config.markdown.enabled {
        crate::commands::markdown::load(&config.markdown)
            .await
            .unwrap_or_else(|e| {
                log::warn!("Failed to load markdown links: {}", e);
                Vec::new()
            })
    } else {
        Vec::new()
    };

    let markdown_commands = filter_ignored(markdown_commands, &config.ignored);
    let scoot_commands =
        filter_ignored(crate::commands::domain::get_scoot_commands(), &config.ignored);

    let mut manager = command_registry
        .lock()
        .map_err(|e| AppError::lock(e))?;
    manager.set_user_commands(commands);
    manager.set_bookmark_commands(bookmarks);
    manager.set_application_commands(app_commands);
    manager.set_scoot_commands(scoot_commands);
    manager.set_markdown_commands(markdown_commands);

    Ok(())
}
