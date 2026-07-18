use crate::config::load_config;
use crate::globals::{F8_LAST_TOGGLE, LAST_VK, MAIN_HWND, TOGGLE_FLAG, WM_HOTKEY_TOGGLE};
use crate::hid::save_and_apply;

const HW_MODE_COUNT: u8 = 7;

pub fn start_keyboard_hook() {
    std::thread::spawn(|| unsafe {
        use windows_sys::Win32::Foundation::HWND;
        use windows_sys::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CallNextHookEx, GetMessageW, MSG, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
            WH_KEYBOARD_LL, WM_KEYDOWN,
        };

        unsafe extern "system" fn hook_proc(code: i32, wparam: usize, lparam: isize) -> isize {
            unsafe {
                if code >= 0 && wparam == WM_KEYDOWN as usize {
                    let kbd = &*(lparam as *const KBDLLHOOKSTRUCT);
                    LAST_VK.store(kbd.vkCode, std::sync::atomic::Ordering::SeqCst);
                    let is_f8 = kbd.vkCode == 0x77 || (kbd.vkCode == 0xFF && kbd.scanCode == 0xE);
                    if is_f8 {
                        let now_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        let last = F8_LAST_TOGGLE.load(std::sync::atomic::Ordering::SeqCst);
                        if now_ms.wrapping_sub(last) < 500 {
                            return CallNextHookEx(0, code, wparam, lparam);
                        }
                        F8_LAST_TOGGLE.store(now_ms, std::sync::atomic::Ordering::SeqCst);
                        let mut config = load_config().unwrap_or_default();
                        config.mode = (config.mode + 1) % HW_MODE_COUNT;
                        let _ = save_and_apply(&config);
                        TOGGLE_FLAG.store(true, std::sync::atomic::Ordering::SeqCst);
                        let h = MAIN_HWND.load(std::sync::atomic::Ordering::SeqCst);
                        if h != 0 {
                            PostMessageW(h as isize as HWND, WM_HOTKEY_TOGGLE, 0, 0);
                        }
                    }
                }
                CallNextHookEx(0, code, wparam, lparam)
            }
        }

        use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
        let hinstance = GetModuleHandleW(std::ptr::null());
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hinstance, 0);
        if hook != 0 {
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {}
            UnhookWindowsHookEx(hook);
        }
    });
}
