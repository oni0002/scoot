use crate::commands::domain::{
    Command, CMD_SCOOT_ADD_COMMAND, CMD_SCOOT_KILL, CMD_SCOOT_OPEN_COMMANDS, CMD_SCOOT_OPEN_LOG,
    CMD_SCOOT_OPEN_SETTINGS, CMD_SCOOT_OPEN_SETTINGS_JSON, CMD_SCOOT_RELOAD,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteResult {
    pub keep_window_open: bool,
}

impl ExecuteResult {
    fn hide() -> Self {
        Self {
            keep_window_open: false,
        }
    }
    fn keep_open() -> Self {
        Self {
            keep_window_open: true,
        }
    }
}

/// Execute a registered command by ID (Tauri command).
/// The webview only names the command; what actually runs is always the registry's copy.
#[tauri::command]
pub async fn execute_command(
    id: String,
    args: Vec<String>,
    state: tauri::State<'_, crate::state::CommandsState>,
    settings_state: tauri::State<'_, crate::state::SettingsState>,
    app_handle: tauri::AppHandle,
) -> Result<ExecuteResult, crate::error::AppError> {
    let command = state
        .commands
        .lock()
        .map_err(crate::error::AppError::lock)?
        .get_command(&id)
        .ok_or_else(|| crate::error::AppError::NotFound(format!("Command not found: {}", id)))?;
    // History is keyed by the command string: ids of external commands are not stable
    let usage_key = command.command.clone();
    let result = run_command(&app_handle, command, args).await?;
    let usage_history = settings_state
        .settings
        .lock()
        .map_err(crate::error::AppError::lock)?
        .usage_history;
    if usage_history {
        state.usage.record(&usage_key).await;
    }
    Ok(result)
}

/// Open a URL or file path typed directly into the search box (Tauri command)
#[tauri::command]
pub async fn open_target(
    target: String,
    app_handle: tauri::AppHandle,
) -> Result<ExecuteResult, crate::error::AppError> {
    let target = target.trim();
    if is_url_scheme_allowed(target) {
        execute_url(&app_handle, target).await?;
    } else {
        execute_local_file(&app_handle, target).await?;
    }
    Ok(ExecuteResult::hide())
}

async fn run_command(
    app_handle: &tauri::AppHandle,
    command: Command,
    args: Vec<String>,
) -> Result<ExecuteResult, crate::error::AppError> {
    // Build command
    let final_command = if command.has_placeholders() {
        let missing = command.missing_slots(&args);
        if !missing.is_empty() {
            return Err(crate::error::AppError::Validation(format!(
                "Missing argument for {}.",
                missing.join(", ")
            )));
        }
        command.substitute_args(&args)
    } else {
        command.command.clone()
    };

    // Scoot command (check by source)
    if command.source == crate::commands::domain::SOURCE_SCOOT
        || final_command.starts_with("scoot://")
    {
        return execute_scoot_command(app_handle, &final_command).await;
    }

    // Other commands - dispatch by category
    let result = match command.category.as_str() {
        crate::commands::domain::CATEGORY_URL => execute_url(app_handle, &final_command).await,
        crate::commands::domain::CATEGORY_FILE => {
            execute_local_file(app_handle, &final_command).await
        }
        crate::commands::domain::CATEGORY_COMMAND => {
            execute_shell_command(
                &final_command,
                &command.working_dir,
                command.show_window.unwrap_or(false),
            )
            .await
        }
        _ => Err(crate::error::AppError::Validation(format!(
            "Unexpected command category: {}",
            command.category
        ))),
    };
    result.map(|_| ExecuteResult::hide())
}

/// Execute internal scoot command
async fn execute_scoot_command(
    app_handle: &tauri::AppHandle,
    command: &str,
) -> Result<ExecuteResult, crate::error::AppError> {
    log::debug!("Executing scoot command: {}", command);

    match command {
        CMD_SCOOT_ADD_COMMAND => {
            crate::lifecycle::open_add_command_dialog(app_handle)?;
            Ok(ExecuteResult::keep_open())
        }
        CMD_SCOOT_OPEN_COMMANDS => {
            crate::commands::ipc::open_commands_json(app_handle.clone()).await?;
            Ok(ExecuteResult::hide())
        }
        CMD_SCOOT_OPEN_SETTINGS_JSON => {
            crate::settings::ipc::open_config_json(app_handle.clone()).await?;
            Ok(ExecuteResult::hide())
        }
        CMD_SCOOT_OPEN_SETTINGS => {
            crate::lifecycle::open_settings_dialog(app_handle)?;
            Ok(ExecuteResult::keep_open())
        }
        CMD_SCOOT_OPEN_LOG => {
            crate::lifecycle::open_log(app_handle)?;
            Ok(ExecuteResult::hide())
        }
        CMD_SCOOT_RELOAD => {
            crate::lifecycle::reload(app_handle).await?;
            Ok(ExecuteResult::keep_open())
        }
        CMD_SCOOT_KILL => {
            crate::lifecycle::quit_app((*app_handle).clone()).await?;
            Ok(ExecuteResult::hide())
        }
        _ => Err(crate::error::AppError::CommandExecution(format!(
            "Unknown scoot command: {}",
            command
        ))),
    }
}

const ALLOWED_URL_SCHEMES: &[&str] = &["http://", "https://", "mailto:", "ftp://"];

pub(crate) fn is_url_scheme_allowed(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    ALLOWED_URL_SCHEMES.iter().any(|s| url_lower.starts_with(s))
}

/// Execute URL command
async fn execute_url(
    app_handle: &tauri::AppHandle,
    url: &str,
) -> Result<(), crate::error::AppError> {
    log::debug!("Opening URL: {}", url);

    if !is_url_scheme_allowed(url) {
        return Err(crate::error::AppError::Validation(format!(
            "URL scheme not allowed: '{}'. Allowed schemes: http, https, mailto, ftp.",
            url
        )));
    }

    crate::os::open_url(app_handle, url).map_err(|e| {
        let error_msg = format!(
            "Failed to open URL '{}': {}. Please check if a web browser or associated application is installed.",
            url, e
        );
        log::error!("Error: {}", error_msg);
        crate::error::AppError::CommandExecution(error_msg)
    })?;

    log::debug!("Successfully opened URL: {}", url);
    Ok(())
}

/// Execute local file
async fn execute_local_file(
    app_handle: &tauri::AppHandle,
    file_path: &str,
) -> Result<(), crate::error::AppError> {
    log::debug!("Opening file: {}", file_path);

    // Expand environment variables
    let expanded_path = crate::os::expand_env_vars(file_path);
    log::debug!("Opening file (expanded): {}", expanded_path);

    // Check if file exists
    if !std::path::Path::new(&expanded_path).exists() {
        let error_msg = if expanded_path == file_path {
            format!("File not found: '{}'", file_path)
        } else {
            format!(
                "File not found: '{}' (expanded from '{}')",
                expanded_path, file_path
            )
        };
        log::error!("Error: {}", error_msg);
        return Err(crate::error::AppError::CommandExecution(error_msg));
    }

    // Open file
    crate::os::open_path(app_handle, &expanded_path).map_err(|e| {
        let error_msg = format!("Failed to open file '{}': {}. Please check file permissions and ensure a default application is set for this file type.", expanded_path, e);
        log::error!("Error: {}", error_msg);
        crate::error::AppError::CommandExecution(error_msg)
    })?;

    log::debug!("Successfully opened file: {}", expanded_path);
    Ok(())
}

/// Execute a shell command
async fn execute_shell_command(
    command: &str,
    working_dir: &Option<String>,
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))] show_window: bool,
) -> Result<(), crate::error::AppError> {
    use std::process::Command as StdCommand;
    log::debug!("Executing shell command: {}", command);

    // If the command is empty, return an error
    if command.trim().is_empty() {
        return Err(crate::error::AppError::Validation(
            "System command cannot be empty.".to_string(),
        ));
    }

    // Build the command
    #[cfg(target_os = "windows")]
    let mut cmd_builder = {
        use std::os::windows::process::CommandExt;
        let mut cmd = StdCommand::new("powershell");
        let mut args = vec!["-NoProfile"];
        if show_window {
            args.push("-NoExit");
        }
        args.push("-Command");
        args.push(command);
        cmd.args(args);
        if show_window {
            cmd.creation_flags(0x00000010); // CREATE_NEW_CONSOLE
        } else {
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        cmd
    };

    #[cfg(not(target_os = "windows"))]
    let mut cmd_builder = {
        let mut cmd = StdCommand::new("sh");
        cmd.args(["-c", command]);
        cmd
    };

    // Set the working directory
    if let Some(dir) = working_dir {
        // Remove double quotes if present
        let trimmed_dir = dir.trim();
        let clean_dir =
            if trimmed_dir.starts_with('"') && trimmed_dir.ends_with('"') && trimmed_dir.len() >= 2
            {
                &trimmed_dir[1..trimmed_dir.len() - 1]
            } else {
                trimmed_dir
            };

        if !clean_dir.is_empty() {
            if std::path::Path::new(clean_dir).exists() {
                cmd_builder.current_dir(clean_dir);
            } else {
                // If the working directory does not exist, issue a warning
                log::warn!(
                    "Warning: Working directory '{}' does not exist. Ignoring.",
                    clean_dir
                );
            }
        }
    }

    // Execute the command asynchronously
    match cmd_builder.spawn() {
        Ok(_) => {
            log::debug!("Command launched successfully (background).");
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to spawn system command '{}': {}.", command, e);
            log::error!("Error: {}", error_msg);
            Err(crate::error::AppError::CommandExecution(error_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_url_schemes_accepted() {
        assert!(is_url_scheme_allowed("https://example.com"));
        assert!(is_url_scheme_allowed("http://example.com"));
        assert!(is_url_scheme_allowed("mailto:user@example.com"));
        assert!(is_url_scheme_allowed("ftp://files.example.com"));
    }

    #[test]
    fn disallowed_url_schemes_rejected() {
        assert!(!is_url_scheme_allowed("javascript:alert(1)"));
        assert!(!is_url_scheme_allowed("file:///C:/Windows/system32"));
        assert!(!is_url_scheme_allowed("scoot://add-command"));
        assert!(!is_url_scheme_allowed("C:\\Users\\foo\\bar.exe"));
        assert!(!is_url_scheme_allowed("data:text/html,<script>"));
    }

    #[test]
    fn url_scheme_check_is_case_insensitive() {
        assert!(is_url_scheme_allowed("HTTPS://example.com"));
        assert!(is_url_scheme_allowed("HTTP://example.com"));
        assert!(!is_url_scheme_allowed("JAVASCRIPT:alert(1)"));
    }
}
