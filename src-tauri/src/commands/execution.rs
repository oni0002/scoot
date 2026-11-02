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
        // 履歴のディスク書き込みを待たせない。応答が遅れるぶん、ウィンドウが閉じるのも遅れる
        let handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            use tauri::Manager;
            handle
                .state::<crate::state::CommandsState>()
                .usage
                .record(&usage_key)
                .await;
        });
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
        execute_url(&app_handle, target, None).await?;
    } else {
        execute_local_file(&app_handle, target, None).await?;
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
    // Decided by the template, not the filled-in text: an argument must not become a Scoot command
    if command.source == crate::commands::domain::SOURCE_SCOOT
        || command.command.starts_with("scoot://")
    {
        return execute_scoot_command(app_handle, &final_command).await;
    }

    // 渡すのは保存済みの固定引数だけ。検索欄の入力は URL の置換にしか入らない
    let open_with = command
        .open_with
        .as_deref()
        .map(|program| (program, command.open_args.as_deref()));

    // Other commands - dispatch by category
    let result = match command.category.as_str() {
        crate::commands::domain::CATEGORY_URL => {
            execute_url(app_handle, &final_command, open_with).await
        }
        crate::commands::domain::CATEGORY_FILE => {
            execute_local_file(app_handle, &final_command, open_with).await
        }
        crate::commands::domain::CATEGORY_COMMAND => {
            execute_shell_command(
                &final_command,
                &command.working_dir,
                command.show_window.unwrap_or(false),
                command.run_as_admin.unwrap_or(false),
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
    open_with: Option<(&str, Option<&str>)>,
) -> Result<(), crate::error::AppError> {
    log::debug!("Opening URL: {}", url);

    if !is_url_scheme_allowed(url) {
        return Err(crate::error::AppError::Validation(format!(
            "URL scheme not allowed: '{}'. Allowed schemes: http, https, mailto, ftp.",
            url
        )));
    }

    if let Some((program, args)) = open_with {
        return crate::os::open_with(program, args, url).inspect_err(|e| log::error!("{}", e));
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
    open_with: Option<(&str, Option<&str>)>,
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

    if let Some((program, args)) = open_with {
        return crate::os::open_with(program, args, &expanded_path)
            .inspect_err(|e| log::error!("{}", e));
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

/// The directory a shell command should start in: quotes and environment variables resolved.
/// None when nothing is set or the directory does not exist, in which case the command runs
/// where Scoot is.
fn resolve_working_dir(dir: &str) -> Option<String> {
    let expanded = crate::os::expand_env_vars(dir.trim());
    let trimmed = expanded.trim();
    let clean = trimmed
        .strip_prefix('"')
        .and_then(|d| d.strip_suffix('"'))
        .unwrap_or(trimmed);
    if clean.is_empty() {
        return None;
    }
    if !std::path::Path::new(clean).exists() {
        log::warn!(
            "Warning: Working directory '{}' does not exist. Ignoring.",
            clean
        );
        return None;
    }
    Some(clean.to_string())
}

/// UTF-16LE の base64。PowerShell の -EncodedCommand が受け取る形式
#[cfg(target_os = "windows")]
fn encode_powershell_script(script: &str) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, b)| acc | (*b as u32) << (16 - 8 * i));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(TABLE[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Start-Process 経由の PowerShell 起動(プログラム名と引数)。
/// ユーザーのコマンドは -EncodedCommand で渡すので、クォートのエスケープが要らない。
/// 窓を出すときは、GUI アプリの子に直接 -NoExit を付けると標準入力が空で即終了するので、
/// 新しいコンソールを持つ Start-Process 経由にする
#[cfg(target_os = "windows")]
fn start_process_launch(
    command: &str,
    working_dir: Option<&str>,
    show_window: bool,
    elevated: bool,
) -> (&'static str, Vec<String>) {
    // Start-Process した先には作業ディレクトリが渡らないので、スクリプトの先頭で移動する。
    // 単一引用符の中では、' と PowerShell が引用符とみなす曲線引用符だけを重ねればよい
    let script = match working_dir {
        Some(dir) => {
            let mut quoted = String::new();
            for c in dir.chars() {
                quoted.push(c);
                if matches!(c, '\'' | '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}') {
                    quoted.push(c);
                }
            }
            format!("Set-Location -LiteralPath '{quoted}'\n{command}")
        }
        None => command.to_string(),
    };
    let mut inner = vec!["'-NoProfile'"];
    if show_window {
        inner.push("'-NoExit'");
    }
    let encoded = format!("'{}'", encode_powershell_script(&script));
    inner.push("'-EncodedCommand'");
    inner.push(&encoded);
    let style = if show_window {
        ""
    } else {
        " -WindowStyle Hidden"
    };
    let verb = if elevated { " -Verb RunAs" } else { "" };
    let start = format!(
        "Start-Process powershell{verb}{style} -ArgumentList {}",
        inner.join(",")
    );
    (
        "powershell",
        vec!["-NoProfile".into(), "-Command".into(), start],
    )
}

/// Execute a shell command
async fn execute_shell_command(
    command: &str,
    working_dir: &Option<String>,
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))] show_window: bool,
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))] run_as_admin: bool,
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
    let dir = working_dir.as_deref().and_then(resolve_working_dir);

    #[cfg(target_os = "windows")]
    let mut cmd_builder = {
        use std::os::windows::process::CommandExt;
        let mut cmd;
        if run_as_admin || show_window {
            let (program, args) =
                start_process_launch(command, dir.as_deref(), show_window, run_as_admin);
            cmd = StdCommand::new(program);
            cmd.args(args);
            // 開いた先のウィンドウは Start-Process が決める。起動役の PowerShell は常に隠す
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        } else {
            cmd = StdCommand::new("powershell");
            cmd.args(["-NoProfile", "-Command", command]);
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

    // Start-Process した先には作業ディレクトリが引き継がれないので、start_process_launch が Set-Location する
    if !(run_as_admin || show_window) {
        if let Some(dir) = &dir {
            cmd_builder.current_dir(dir);
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
    fn working_dir_drops_surrounding_quotes_and_blanks() {
        let dir = std::env::temp_dir().to_string_lossy().into_owned();
        assert_eq!(resolve_working_dir(&dir).as_deref(), Some(dir.as_str()));
        assert_eq!(
            resolve_working_dir(&format!("  \"{}\" ", dir)).as_deref(),
            Some(dir.as_str())
        );
        assert_eq!(resolve_working_dir(""), None);
        assert_eq!(resolve_working_dir("  "), None);
        assert_eq!(resolve_working_dir("\"\""), None);
        assert_eq!(resolve_working_dir("\""), None);
    }

    #[test]
    fn working_dir_that_does_not_exist_is_ignored() {
        assert_eq!(resolve_working_dir(r"Z:\no\such\dir-scoot"), None);
    }

    #[test]
    fn url_scheme_check_is_case_insensitive() {
        assert!(is_url_scheme_allowed("HTTPS://example.com"));
        assert!(is_url_scheme_allowed("HTTP://example.com"));
        assert!(!is_url_scheme_allowed("JAVASCRIPT:alert(1)"));
    }

    #[cfg(target_os = "windows")]
    fn decode_powershell_script(b64: &str) -> String {
        let value = |c: u8| match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            _ => 63,
        };
        let mut bytes = Vec::new();
        for chunk in b64.as_bytes().chunks(4) {
            let data: Vec<u8> = chunk.iter().copied().take_while(|c| *c != b'=').collect();
            let n = data
                .iter()
                .enumerate()
                .fold(0u32, |acc, (i, c)| acc | (value(*c) as u32) << (18 - 6 * i));
            bytes.extend((0..data.len() - 1).map(|i| (n >> (16 - 8 * i)) as u8));
        }
        let units: Vec<u16> = bytes
            .chunks(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        String::from_utf16(&units).unwrap()
    }

    #[cfg(target_os = "windows")]
    fn encoded_script(args: &[String]) -> String {
        let b64 = args[2].rsplit("'-EncodedCommand','").next().unwrap();
        decode_powershell_script(b64.trim_end_matches('\''))
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn elevated_launch_round_trips_awkward_commands() {
        for command in [
            "echo hi",
            "echo \"a 'b' `c` $d $(x)\"; 1",
            "echo \"日本語 \u{1F600}\"",
            "a",
            "ab",
        ] {
            let (program, args) = start_process_launch(command, None, true, true);
            assert_eq!(program, "powershell");
            assert_eq!(encoded_script(&args), command);
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn elevated_launch_moves_to_the_working_dir_inside_the_script() {
        let (_, args) = start_process_launch("dir", Some("C:\\it's\\\u{2019}x"), true, true);
        assert_eq!(
            encoded_script(&args),
            "Set-Location -LiteralPath 'C:\\it''s\\\u{2019}\u{2019}x'\ndir"
        );
        let (_, args) = start_process_launch("dir", None, true, true);
        assert_eq!(encoded_script(&args), "dir");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn elevated_launch_shows_or_hides_the_window() {
        let (_, shown) = start_process_launch("dir", None, true, true);
        assert!(shown[2].starts_with("Start-Process powershell -Verb RunAs -ArgumentList"));
        assert!(shown[2].contains("'-NoExit'"));
        let (_, hidden) = start_process_launch("dir", None, false, true);
        assert!(hidden[2].contains("-Verb RunAs -WindowStyle Hidden -ArgumentList"));
        assert!(!hidden[2].contains("-NoExit"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn start_process_launch_asks_for_uac_only_when_elevated() {
        let (_, plain) = start_process_launch("dir", None, true, false);
        assert!(!plain[2].contains("-Verb RunAs"));
        assert!(plain[2].starts_with("Start-Process powershell -ArgumentList"));
        let (_, elevated) = start_process_launch("dir", None, true, true);
        assert!(elevated[2].contains("-Verb RunAs"));
    }
}
