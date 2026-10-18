//! Windows shell links (.lnk): writing one, and reading where one points. Only the COM calls live
//! here; what to do with a link is up to the caller.

use crate::error::AppError;
use std::path::{Path, PathBuf};

/// Write a shortcut at `path` that starts `target` in `working_dir`, replacing any existing one.
#[cfg(target_os = "windows")]
pub fn create(path: &Path, target: &Path, working_dir: &Path) -> Result<(), AppError> {
    use windows::core::{Interface, HSTRING};
    use windows::Win32::System::Com::{CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    com::with(|| unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        link.SetPath(&HSTRING::from(target.as_os_str()))?;
        link.SetWorkingDirectory(&HSTRING::from(working_dir.as_os_str()))?;
        let file: IPersistFile = link.cast()?;
        file.Save(&HSTRING::from(path.as_os_str()), true)
    })
    .map_err(|e| AppError::System(format!("Failed to create the shortcut: {e}")))
}

/// The file a shortcut points at, or `None` when it points at nothing a path can name (a Store
/// app, say). It reads what is stored and does not search for a target that has moved.
#[cfg(target_os = "windows")]
pub fn target(path: &Path) -> Result<Option<PathBuf>, AppError> {
    use windows::core::{Interface, HSTRING};
    use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
    use windows::Win32::System::Com::{
        CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER, STGM_READ,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    com::with(|| unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        let file: IPersistFile = link.cast()?;
        file.Load(&HSTRING::from(path.as_os_str()), STGM_READ)?;
        let mut buffer = [0u16; 1024];
        let mut find = WIN32_FIND_DATAW::default();
        link.GetPath(&mut buffer, &mut find, 0)?;
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        Ok((len > 0).then(|| PathBuf::from(String::from_utf16_lossy(&buffer[..len]))))
    })
    .map_err(|e| AppError::System(format!("Failed to read the shortcut: {e}")))
}

#[cfg(not(target_os = "windows"))]
pub fn create(_: &Path, _: &Path, _: &Path) -> Result<(), AppError> {
    Err(AppError::System(
        "Shortcuts are only supported on Windows".to_string(),
    ))
}

#[cfg(not(target_os = "windows"))]
pub fn target(_: &Path) -> Result<Option<PathBuf>, AppError> {
    Err(AppError::System(
        "Shortcuts are only supported on Windows".to_string(),
    ))
}

#[cfg(target_os = "windows")]
mod com {
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

    /// Run `f` on a thread with COM initialised, and balance it afterwards. COM is only
    /// uninitialised if this call initialised it.
    pub fn with<T>(f: impl FnOnce() -> windows::core::Result<T>) -> windows::core::Result<T> {
        struct Guard;
        impl Drop for Guard {
            fn drop(&mut self) {
                unsafe { CoUninitialize() };
            }
        }

        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()? };
        let _guard = Guard;
        f()
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("scoot-link-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn reads_back_the_target_it_was_given() {
        let dir = temp_dir("target");
        let link = dir.join("x.lnk");
        let exe = std::env::current_exe().unwrap();

        create(&link, &exe, exe.parent().unwrap()).unwrap();
        assert_eq!(target(&link).unwrap(), Some(exe));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_file_that_is_not_a_shortcut_is_an_error() {
        let dir = temp_dir("bogus");
        std::fs::create_dir_all(&dir).unwrap();
        let bogus = dir.join("x.lnk");
        std::fs::write(&bogus, b"not a shortcut").unwrap();

        assert!(target(&bogus).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
