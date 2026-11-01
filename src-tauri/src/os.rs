use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;

static ENV_VAR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"%([^%]+)%").expect("invalid regex"));
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

/// Open the specified path (file, directory, URL) with the default application
pub fn open_path(app_handle: &AppHandle, path: &str) -> Result<(), crate::error::AppError> {
    app_handle
        .opener()
        .open_path(path, None::<&str>)
        .map_err(|e| {
            crate::error::AppError::System(format!("Failed to open path '{}': {}", path, e))
        })
}

/// Open the specified URL with the default browser
pub fn open_url(app_handle: &AppHandle, url: &str) -> Result<(), crate::error::AppError> {
    app_handle
        .opener()
        .open_url(url, None::<&str>)
        .map_err(|e| crate::error::AppError::System(format!("Failed to open URL '{}': {}", url, e)))
}

/// 「開くプログラム」と、その生の引数文字列(固定引数 + 引用符で囲んだ対象)。
/// 対象は Command 自身の値から来る。`"` は引用符を閉じられないよう潰す。
pub fn open_with_command_line(
    program: &str,
    open_args: Option<&str>,
    target: &str,
) -> (String, String) {
    let target = target.replace('"', "%22");
    let line = match open_args.map(str::trim).filter(|a| !a.is_empty()) {
        Some(args) => format!("{args} \"{target}\""),
        None => format!("\"{target}\""),
    };
    (expand_env_vars(program.trim()), line)
}

/// 指定したプログラムで対象を開く。コンソールは出さず、終了を待たない
pub fn open_with(
    program: &str,
    open_args: Option<&str>,
    target: &str,
) -> Result<(), crate::error::AppError> {
    let (program, line) = open_with_command_line(program, open_args, target);
    let mut cmd = std::process::Command::new(&program);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // 引数を分割し直さず、そのまま渡す
        cmd.raw_arg(&line).creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    #[cfg(not(target_os = "windows"))]
    {
        cmd.args(
            open_args
                .unwrap_or_default()
                .split_whitespace()
                .chain(std::iter::once(target)),
        );
    }

    cmd.spawn().map(|_| ()).map_err(|e| {
        crate::error::AppError::CommandExecution(format!(
            "Failed to open with '{}': {}.",
            program, e
        ))
    })
}

/// The directory holding the exe. Scoot is portable: settings.json, commands.json and logs/ live
/// next to it.
pub fn app_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// The log directory
pub fn log_dir() -> PathBuf {
    app_dir().join("logs")
}

/// `path` with a leading `~` replaced by the user profile. Only `~` alone or before a separator:
/// `PROGRA~1` and `~name` are not home directories.
fn home_prefix(path: &str) -> Option<String> {
    let rest = path.strip_prefix('~')?;
    if !(rest.is_empty() || rest.starts_with(['/', '\\'])) {
        return None;
    }
    let home = std::env::var("USERPROFILE").ok()?;
    Some(format!("{home}{rest}"))
}

/// Expand Windows environment variables like %APPDATA%, and a leading `~` to the user profile
pub fn expand_env_vars(path: &str) -> String {
    let home = home_prefix(path);
    let path = home.as_deref().unwrap_or(path);
    ENV_VAR_RE
        .replace_all(path, |caps: &regex::Captures| {
            let key = &caps[1];
            std::env::var(key).unwrap_or_else(|_| format!("%{}%", key))
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_expand_home() {
        env::set_var("USERPROFILE", r"C:\Users\me");
        assert_eq!(expand_env_vars("~"), r"C:\Users\me");
        assert_eq!(expand_env_vars("~/docs"), r"C:\Users\me/docs");
        assert_eq!(expand_env_vars(r"~\docs"), r"C:\Users\me\docs");
        assert_eq!(expand_env_vars("~name/x"), "~name/x");
        assert_eq!(expand_env_vars("a/~/b"), "a/~/b");
    }

    #[test]
    fn test_expand_existing_var() {
        let key = "SCOOT_TEST_VAR";
        let value = "test_value";
        env::set_var(key, value);

        let input = format!("path/to/%{}%/file", key);
        let expected = format!("path/to/{}/file", value);

        assert_eq!(expand_env_vars(&input), expected);

        env::remove_var(key);
    }

    #[test]
    fn test_expand_non_existing_var() {
        let input = "path/to/%NON_EXISTING_VAR%/file";
        assert_eq!(expand_env_vars(input), input);
    }

    #[test]
    fn open_with_line_quotes_the_target_after_the_fixed_args() {
        let (program, line) = open_with_command_line(
            " chrome.exe ",
            Some("--incognito --profile-directory=\"Profile 1\""),
            "https://example.com/a b",
        );
        assert_eq!(program, "chrome.exe");
        assert_eq!(
            line,
            "--incognito --profile-directory=\"Profile 1\" \"https://example.com/a b\""
        );
    }

    #[test]
    fn open_with_line_without_args_is_just_the_target() {
        for args in [None, Some(""), Some("  ")] {
            let (_, line) = open_with_command_line("notepad.exe", args, r"C:\a b\c.txt");
            assert_eq!(line, r#""C:\a b\c.txt""#);
        }
    }

    #[test]
    fn open_with_line_cannot_close_the_quote_early() {
        let (_, line) = open_with_command_line("p.exe", None, r#"https://x.com/"&calc"#);
        assert_eq!(line, r#""https://x.com/%22&calc""#);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn open_with_really_spawns_the_program() {
        // 空白入りのパスでも、固定引数と対象がそのまま届いてコピーされる
        let dir = std::env::temp_dir().join("scoot open with test");
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("in.txt");
        let dst = dir.join("out.txt");
        std::fs::write(&src, "hello").unwrap();
        let _ = std::fs::remove_file(&dst);
        let args = format!("/c copy /y \"{}\"", src.display());
        open_with("cmd.exe", Some(&args), &dst.to_string_lossy()).unwrap();
        for _ in 0..50 {
            if dst.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "hello");
    }

    #[test]
    fn test_no_vars() {
        let input = "path/to/file";
        assert_eq!(expand_env_vars(input), input);
    }
}
