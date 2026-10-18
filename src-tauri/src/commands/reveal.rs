//! Show where a file Command lives: Explorer opens with the file selected.

use crate::commands::domain::{Command, CATEGORY_FILE};
use crate::error::AppError;
use std::path::{Path, PathBuf};

/// Reveal a registered file Command in Explorer (Tauri command). Like `execute_command`, the
/// webview only names the Command; the path always comes from the registry's copy.
#[tauri::command]
pub async fn reveal_command(
    id: String,
    state: tauri::State<'_, crate::state::CommandsState>,
) -> Result<(), AppError> {
    let command = state
        .commands
        .lock()
        .map_err(AppError::lock)?
        .get_command(&id)
        .ok_or_else(|| AppError::NotFound(format!("Command not found: {id}")))?;
    let path = revealable_path(&command)?;

    tokio::task::spawn_blocking(move || reveal(&location(&path)))
        .await
        .map_err(|e| AppError::System(format!("Reveal task failed: {e}")))?
}

/// The path of a Command that has a place to show. Only a plain file or folder does: a URL or a
/// shell command is not somewhere, and a path with placeholders is not one until it is filled in.
fn revealable_path(command: &Command) -> Result<PathBuf, AppError> {
    if command.category != CATEGORY_FILE || command.has_placeholders() {
        return Err(AppError::Validation(format!(
            "\"{}\" has no location to show.",
            command.name
        )));
    }
    let path = PathBuf::from(crate::os::expand_env_vars(&command.command));
    if !path.exists() {
        return Err(AppError::NotFound(format!(
            "File not found: '{}'",
            path.display()
        )));
    }
    Ok(path)
}

/// What to select in Explorer. A shortcut stands for the file it starts, so show that file; one
/// that points at nothing a path can name (a Store app) is shown as itself.
fn location(path: &Path) -> PathBuf {
    let is_shortcut = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("lnk"));
    if !is_shortcut {
        return path.to_path_buf();
    }
    match crate::shell_link::target(path) {
        Ok(Some(target)) if target.exists() => target,
        Ok(_) => path.to_path_buf(),
        Err(e) => {
            log::warn!("Could not resolve '{}': {}", path.display(), e);
            path.to_path_buf()
        }
    }
}

/// The argument Explorer takes to open with `path` selected. The quotes are part of the argument,
/// so it is passed through unescaped; Windows paths cannot contain a quote themselves.
fn select_argument(path: &Path) -> String {
    format!("/select,\"{}\"", path.display())
}

#[cfg(target_os = "windows")]
fn reveal(path: &Path) -> Result<(), AppError> {
    use std::os::windows::process::CommandExt;

    // Explorer reports failure through its exit code even when it worked, so only a failure to
    // start it is an error.
    std::process::Command::new("explorer.exe")
        .raw_arg(select_argument(path))
        .spawn()
        .map(drop)
        .map_err(|e| {
            AppError::CommandExecution(format!(
                "Failed to open '{}' in Explorer: {e}",
                path.display()
            ))
        })
}

#[cfg(not(target_os = "windows"))]
fn reveal(_: &Path) -> Result<(), AppError> {
    Err(AppError::System(
        "Revealing a file is only supported on Windows".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::domain::{CATEGORY_COMMAND, CATEGORY_URL, SOURCE_USER};

    fn command(category: &str, command: &str) -> Command {
        Command::new(
            "x".to_string(),
            category,
            SOURCE_USER,
            command.to_string(),
            String::new(),
        )
    }

    #[test]
    fn only_an_existing_file_command_can_be_revealed() {
        let exe = std::env::current_exe().unwrap();
        let ok = revealable_path(&command(CATEGORY_FILE, &exe.to_string_lossy()));
        assert_eq!(ok.unwrap(), exe);

        assert!(revealable_path(&command(CATEGORY_URL, "https://example.com")).is_err());
        assert!(revealable_path(&command(CATEGORY_COMMAND, "git status")).is_err());
        assert!(revealable_path(&command(CATEGORY_FILE, r"C:\no\such\file.txt")).is_err());
    }

    #[test]
    fn a_path_with_placeholders_is_not_a_location() {
        let exe = std::env::current_exe().unwrap();
        let with_slot = format!("{}{{$1}}", exe.display());
        assert!(revealable_path(&command(CATEGORY_FILE, &with_slot)).is_err());
    }

    #[test]
    fn explorer_gets_the_path_quoted() {
        assert_eq!(
            select_argument(Path::new(r"C:\Program Files\App\app.exe")),
            r#"/select,"C:\Program Files\App\app.exe""#
        );
    }

    #[test]
    fn a_plain_file_is_shown_as_itself() {
        let exe = std::env::current_exe().unwrap();
        assert_eq!(location(&exe), exe);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn a_shortcut_is_shown_as_the_file_it_starts() {
        let dir = std::env::temp_dir().join(format!("scoot-reveal-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let exe = std::env::current_exe().unwrap();
        let link = dir.join("app.lnk");
        crate::shell_link::create(&link, &exe, exe.parent().unwrap()).unwrap();

        assert_eq!(location(&link), exe);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_shortcut_that_cannot_be_read_is_shown_as_itself() {
        let dir = std::env::temp_dir().join(format!("scoot-reveal-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bogus = dir.join("x.lnk");
        std::fs::write(&bogus, b"not a shortcut").unwrap();

        assert_eq!(location(&bogus), bogus);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
