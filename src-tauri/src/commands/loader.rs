use crate::commands::domain::Command;
use crate::commands::registry::CommandRegistry;
use crate::error::AppError;
use std::collections::{BTreeMap, HashSet};
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
    let mut external: Vec<Command> = bookmarks
        .into_iter()
        .chain(app_commands)
        .chain(markdown_commands)
        .chain(crate::commands::domain::get_scoot_commands())
        .filter(|c| !ignored.contains(c.command.as_str()))
        .collect();
    let user_aliases: HashSet<&str> = commands.iter().filter_map(|c| c.alias.as_deref()).collect();
    apply_aliases(&mut external, &config.aliases, &user_aliases);

    let mut manager = command_registry.lock().map_err(AppError::lock)?;
    // External first: user commands are checked against their aliases, and `apply_aliases` has
    // already given way to every alias a user command holds.
    manager.set_external_commands(external);
    manager.set_user_commands(commands);

    Ok(())
}

/// Give external commands the aliases saved in config.json. A hand-edited file can hold an alias
/// twice, or one a user command already has; the user command wins, then the first external one.
fn apply_aliases(
    external: &mut [Command],
    aliases: &BTreeMap<String, String>,
    user_aliases: &HashSet<&str>,
) {
    let mut taken: HashSet<&str> = user_aliases.clone();
    for (command, alias) in aliases {
        if !external.iter().any(|c| &c.command == command) {
            continue;
        }
        if !taken.insert(alias.as_str()) {
            log::warn!(
                "Alias '{}' for '{}' is already in use; skipped.",
                alias,
                command
            );
            continue;
        }
        for c in external.iter_mut().filter(|c| &c.command == command) {
            c.alias = Some(alias.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn external(command: &str) -> Command {
        Command::new("n", "file", "application", command, "")
    }

    #[test]
    fn aliases_land_on_matching_commands_and_skip_clashes() {
        let mut cmds = vec![external("a"), external("b"), external("c")];
        let aliases = BTreeMap::from([
            ("a".to_string(), "x".to_string()),
            ("b".to_string(), "x".to_string()), // same alias as "a": skipped
            ("c".to_string(), "mine".to_string()), // a user command has it: skipped
            ("gone".to_string(), "y".to_string()), // no such command: ignored
        ]);
        apply_aliases(&mut cmds, &aliases, &HashSet::from(["mine"]));
        assert_eq!(cmds[0].alias.as_deref(), Some("x"));
        assert_eq!(cmds[1].alias, None);
        assert_eq!(cmds[2].alias, None);
    }
}
