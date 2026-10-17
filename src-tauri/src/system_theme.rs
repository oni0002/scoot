//! The theme of the Windows taskbar, where the tray lives. This is the "Windows mode" setting,
//! which can differ from the "app mode" that the webview follows.

/// Whether the taskbar uses the light theme. Defaults to dark, as Windows does.
pub fn taskbar_is_light() -> bool {
    imp::taskbar_is_light()
}

/// Calls `on_change` with the new value whenever the taskbar theme changes.
pub fn watch_taskbar_theme(on_change: impl Fn(bool) + Send + 'static) {
    imp::watch(on_change);
}

#[cfg(target_os = "windows")]
mod imp {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegGetValueW, RegNotifyChangeKeyValue, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER,
        KEY_READ, REG_NOTIFY_CHANGE_LAST_SET, RRF_RT_REG_DWORD,
    };

    const PERSONALIZE: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    const VALUE: &str = "SystemUsesLightTheme";

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    struct Key(HKEY);

    impl Drop for Key {
        fn drop(&mut self) {
            unsafe { RegCloseKey(self.0) };
        }
    }

    fn open() -> Option<Key> {
        let mut key: HKEY = null_mut();
        let path = wide(PERSONALIZE);
        let status =
            unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, path.as_ptr(), 0, KEY_READ, &mut key) };
        (status == ERROR_SUCCESS).then_some(Key(key))
    }

    fn read(key: &Key) -> bool {
        let name = wide(VALUE);
        let mut value: u32 = 0;
        let mut size = size_of::<u32>() as u32;
        let status = unsafe {
            RegGetValueW(
                key.0,
                null(),
                name.as_ptr(),
                RRF_RT_REG_DWORD,
                null_mut(),
                (&mut value as *mut u32).cast(),
                &mut size,
            )
        };
        status == ERROR_SUCCESS && value != 0
    }

    pub fn taskbar_is_light() -> bool {
        open().is_some_and(|key| read(&key))
    }

    pub fn watch(on_change: impl Fn(bool) + Send + 'static) {
        std::thread::spawn(move || {
            let Some(key) = open() else {
                log::warn!(
                    "Cannot open the Personalize key; the tray icon will not follow the theme"
                );
                return;
            };
            let mut light = read(&key);
            // Blocks until a value under the key changes, so the thread costs nothing while idle.
            while unsafe {
                RegNotifyChangeKeyValue(key.0, 0, REG_NOTIFY_CHANGE_LAST_SET, null_mut(), 0)
            } == ERROR_SUCCESS
            {
                let now = read(&key);
                if now != light {
                    light = now;
                    on_change(light);
                }
            }
        });
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    pub fn taskbar_is_light() -> bool {
        false
    }
    pub fn watch(_on_change: impl Fn(bool) + Send + 'static) {}
}
