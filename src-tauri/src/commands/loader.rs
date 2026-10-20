use crate::commands::domain::Command;
use crate::commands::registry::CommandRegistry;
use crate::error::AppError;
use std::collections::{BTreeMap, HashSet};
use tauri::Runtime;

/// Reload commands, bookmarks, and apps into the registry
pub async fn reload<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    command_store: &crate::commands::store::CommandStore,
    command_registry: &std::sync::Mutex<CommandRegistry>,
    settings: &crate::settings::domain::Settings,
) -> Result<(), AppError> {
    log::debug!("Loading commands, bookmarks, applications, and markdown links in parallel");

    let (commands_result, bookmarks, app_commands, markdown_commands) = tokio::join!(
        command_store.load(),
        async {
            if settings.bookmarks.enabled {
                crate::commands::bookmark::load(&settings.bookmarks)
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
            if settings.applications.enabled {
                crate::commands::application::load(
                    &settings.applications.directories,
                    &settings.applications.extensions,
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
            if settings.markdown.enabled {
                crate::commands::markdown::load(&settings.markdown)
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

    // None: commands.json could not be read, so the commands already loaded stay as they are
    let commands = match commands_result {
        Ok(mut cmds) => {
            crate::notices::clear(app_handle, "commands");
            for cmd in &mut cmds {
                cmd.source = crate::commands::domain::SOURCE_USER.to_string();
            }
            Some(cmds)
        }
        Err(e) => {
            let msg = format!("Failed to load commands.json: {}", e);
            log::error!("{}", msg);
            crate::notices::raise(app_handle, "commands", crate::notices::ERROR, msg);
            None
        }
    };

    let ignored: HashSet<&str> = settings.ignored.iter().map(String::as_str).collect();
    let mut external: Vec<Command> = bookmarks
        .into_iter()
        .chain(app_commands)
        .chain(markdown_commands)
        .chain(crate::commands::domain::get_scoot_commands())
        .filter(|c| !ignored.contains(c.command.as_str()))
        .collect();
    let mut manager = command_registry.lock().map_err(AppError::lock)?;
    let user_aliases: HashSet<String> = match &commands {
        Some(commands) => commands.iter().filter_map(|c| c.alias.clone()).collect(),
        None => manager.user_aliases(),
    };
    let user_aliases: HashSet<&str> = user_aliases.iter().map(String::as_str).collect();
    apply_aliases(&mut external, &settings.aliases, &user_aliases);

    // External first: user commands are checked against their aliases, and `apply_aliases` has
    // already given way to every alias a user command holds.
    manager.set_external_commands(external);
    manager.set_load_failed(commands.is_none());
    if let Some(commands) = commands {
        manager.set_user_commands(commands);
    }

    Ok(())
}

/// Give external commands the aliases saved in settings.json. A hand-edited file can hold an alias
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
