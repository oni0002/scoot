use tauri::AppHandle;

#[tauri::command]
pub fn start_hotkey_capture(app: AppHandle) {
    imp::start(app);
}

#[tauri::command]
pub fn stop_hotkey_capture() {
    imp::stop();
}

#[cfg(target_os = "windows")]
mod imp {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Mutex;
    use tauri::{AppHandle, Emitter};
    use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
        UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN,
        WM_QUIT, WM_SYSKEYDOWN,
    };

    const LLKHF_ALTDOWN: u32 = 0x20;

    static CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);
    static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);
    static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);

    unsafe extern "system" fn keyboard_hook(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if n_code >= 0 && CAPTURE_ACTIVE.load(Ordering::SeqCst) {
            let msg = w_param as u32;
            if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                let kb = &*(l_param as *const KBDLLHOOKSTRUCT);
                let vk = kb.vkCode as u16;

                let is_modifier = vk == VK_CONTROL
                    || vk == VK_SHIFT
                    || vk == VK_MENU
                    || vk == VK_LWIN
                    || vk == VK_RWIN;

                if !is_modifier {
                    let alt = (kb.flags & LLKHF_ALTDOWN) != 0;
                    let ctrl = (GetKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0;
                    let shift = (GetKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0;
                    let win = (GetKeyState(VK_LWIN as i32) as u16 & 0x8000) != 0
                        || (GetKeyState(VK_RWIN as i32) as u16 & 0x8000) != 0;

                    if ctrl || alt || shift || win {
                        if let Some(key_name) = vk_to_key_name(vk) {
                            let mut parts: Vec<&str> = Vec::new();
                            if ctrl {
                                parts.push("Ctrl");
                            }
                            if alt {
                                parts.push("Alt");
                            }
                            if shift {
                                parts.push("Shift");
                            }
                            if win {
                                parts.push("Super");
                            }

                            let hotkey = format!("{}+{}", parts.join("+"), key_name);
                            log::debug!("Hotkey captured: {}", hotkey);

                            CAPTURE_ACTIVE.store(false, Ordering::SeqCst);

                            if let Ok(guard) = APP_HANDLE.lock() {
                                if let Some(handle) = guard.as_ref() {
                                    let _ = handle.emit("hotkey-captured", hotkey);
                                }
                            }

                            let tid = HOOK_THREAD_ID.load(Ordering::SeqCst);
                            if tid != 0 {
                                PostThreadMessageW(tid, WM_QUIT, 0, 0);
                            }
                        }
                    }
                }
            }
        }
        CallNextHookEx(0, n_code, w_param, l_param)
    }

    fn vk_to_key_name(vk: u16) -> Option<String> {
        match vk {
            0x20 => Some("Space".to_string()),
            0x0D => Some("Enter".to_string()),
            0x09 => Some("Tab".to_string()),
            0x30..=0x39 => Some((vk as u8 as char).to_string()),
            0x41..=0x5A => Some((vk as u8 as char).to_string()),
            0x70..=0x7B => Some(format!("F{}", vk - 0x70 + 1)),
            _ => None,
        }
    }

    pub fn start(app: AppHandle) {
        if CAPTURE_ACTIVE.load(Ordering::SeqCst) {
            return;
        }

        if let Ok(mut guard) = APP_HANDLE.lock() {
            *guard = Some(app);
        }

        CAPTURE_ACTIVE.store(true, Ordering::SeqCst);

        std::thread::spawn(|| unsafe {
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), 0, 0);
            if hook == 0 {
                log::error!("Failed to install low-level keyboard hook");
                CAPTURE_ACTIVE.store(false, Ordering::SeqCst);
                return;
            }

            let tid = GetCurrentThreadId();
            HOOK_THREAD_ID.store(tid, Ordering::SeqCst);

            // Exit early if capture was cancelled before the thread started
            if !CAPTURE_ACTIVE.load(Ordering::SeqCst) {
                UnhookWindowsHookEx(hook);
                HOOK_THREAD_ID.store(0, Ordering::SeqCst);
                return;
            }

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {}

            UnhookWindowsHookEx(hook);
            HOOK_THREAD_ID.store(0, Ordering::SeqCst);
        });
    }

    pub fn stop() {
        CAPTURE_ACTIVE.store(false, Ordering::SeqCst);
        let tid = HOOK_THREAD_ID.load(Ordering::SeqCst);
        if tid != 0 {
            unsafe {
                PostThreadMessageW(tid, WM_QUIT, 0, 0);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use tauri::AppHandle;

    pub fn start(_app: AppHandle) {}
    pub fn stop() {}
}
