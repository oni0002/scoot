use tauri::AppHandle;

#[tauri::command]
pub fn start_hotkey_capture(app: AppHandle) {
    crate::shortcut::unregister(&app);
    imp::start(app);
}

#[tauri::command]
pub async fn stop_hotkey_capture(app: AppHandle) {
    imp::stop();
    use tauri::Manager;
    let hotkey = if let Some(state) = app.try_state::<crate::state::ConfigState>() {
        if let Ok(config) = state.config.lock() {
            config.hotkey.clone()
        } else {
            crate::config::domain::DEFAULT_SHORTCUT.to_string()
        }
    } else {
        crate::config::domain::DEFAULT_SHORTCUT.to_string()
    };
    if let Err(e) = crate::shortcut::setup_global_shortcuts(&app, &hotkey) {
        log::error!("Failed to re-register shortcut after hotkey capture: {}", e);
    }
}

#[cfg(target_os = "windows")]
pub unsafe fn setup_window_subclass(hwnd: *mut core::ffi::c_void) {
    imp::setup_subclass(hwnd);
}

#[cfg(target_os = "windows")]
mod imp {
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
    use std::sync::Mutex;
    use tauri::{AppHandle, Emitter};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, SC_KEYMENU,
        WM_SYSCOMMAND, WNDPROC,
    };

    // True while the hotkey capture UI is active.
    pub static CAPTURING: AtomicBool = AtomicBool::new(false);
    // Prevents duplicate emit if both DOM and SC_KEYMENU fire for the same keypress.
    static HOTKEY_EMITTED: AtomicBool = AtomicBool::new(false);
    static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
    static ORIGINAL_WNDPROC: AtomicIsize = AtomicIsize::new(0);

    fn clone_handle() -> Option<AppHandle> {
        APP_HANDLE.lock().ok().and_then(|g| g.as_ref().cloned())
    }

    // ── Window subclass ──────────────────────────────────────────────────────
    // Capture path for Alt+<key> combos that generate WM_SYSCOMMAND SC_KEYMENU
    // instead of reaching the WebView2 DOM (notably Alt+Space, and any Alt+key
    // that triggers menu-mode handling before WebView2 can deliver the event).

    unsafe extern "system" fn subclass_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_SYSCOMMAND
            && wparam & 0xFFF0 == SC_KEYMENU as usize
            && CAPTURING.load(Ordering::SeqCst)
        {
            // Only emit once per capture session; DOM may have already emitted.
            if !HOTKEY_EMITTED.swap(true, Ordering::SeqCst) {
                let key_name = sc_keymenu_key_name(lparam);
                let hotkey = format!("Alt+{}", key_name);
                if let Some(handle) = clone_handle() {
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = handle.emit("hotkey-captured", hotkey) {
                            log::error!("subclass: emit failed: {}", e);
                        }
                    });
                }
            }
            return 0; // suppress system menu regardless
        }

        let orig = ORIGINAL_WNDPROC.load(Ordering::SeqCst);
        if orig != 0 {
            let orig_fn: WNDPROC = Some(std::mem::transmute(orig));
            CallWindowProcW(orig_fn, hwnd, msg, wparam, lparam)
        } else {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }

    // Maps the SC_KEYMENU lparam (a character code) to a human-readable key name.
    fn sc_keymenu_key_name(lparam: LPARAM) -> String {
        match lparam as u32 {
            0x20 => "Space".to_string(),
            0x0D => "Enter".to_string(),
            0x09 => "Tab".to_string(),
            c @ 0x30..=0x39 => (c as u8 as char).to_string(), // 0–9
            c @ 0x41..=0x5A => (c as u8 as char).to_string(), // A–Z
            c @ 0x61..=0x7A => ((c as u8 - 0x20) as char).to_string(), // a–z → uppercase
            _ => format!("{:#x}", lparam),
        }
    }

    pub unsafe fn setup_subclass(hwnd: HWND) {
        let orig = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, subclass_wndproc as isize);
        ORIGINAL_WNDPROC.store(orig, Ordering::SeqCst);
    }

    pub fn start(app: AppHandle) {
        CAPTURING.store(true, Ordering::SeqCst);
        HOTKEY_EMITTED.store(false, Ordering::SeqCst);
        if let Ok(mut guard) = APP_HANDLE.lock() {
            *guard = Some(app);
        }
    }

    pub fn stop() {
        CAPTURING.store(false, Ordering::SeqCst);
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use tauri::AppHandle;
    pub fn start(_app: AppHandle) {}
    pub fn stop() {}
}
