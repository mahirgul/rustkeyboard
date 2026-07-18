use std::sync::atomic::Ordering;

use crate::globals::{MAIN_HWND, WM_TRAYICON};

pub fn start_tray_icon_thread() {
    std::thread::spawn(|| {
        fn dbg_log(msg: &str) {
            use std::io::Write;
            let log_path = format!("{}\\tray_debug.log", crate::config::get_app_dir());
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
            {
                let _ = writeln!(f, "[{:?}] {}", std::time::SystemTime::now(), msg);
            }
        }

        unsafe {
            use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
            use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
            use windows_sys::Win32::UI::Shell::{
                NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
                Shell_NotifyIconW,
            };
            use windows_sys::Win32::UI::WindowsAndMessaging::*;

            dbg_log("Tray thread started");

            // Window class name - static so it won't be dropped
            static CLASS_NAME_BYTES: &[u16] = &[
                b'R' as u16,
                b'K' as u16,
                b'T' as u16,
                b'r' as u16,
                b'a' as u16,
                b'y' as u16,
                0,
            ];

            unsafe extern "system" fn tray_wnd_proc(
                hwnd: HWND,
                msg: u32,
                wparam: WPARAM,
                lparam: LPARAM,
            ) -> LRESULT {
                unsafe {
                    if msg == WM_TRAYICON {
                        // On newer Windows, LOWORD(lparam) is the event
                        let event = (lparam & 0xFFFF) as u32;

                        fn show_main_window() {
                            // Retry up to 20 times (2 seconds) waiting for MAIN_HWND
                            for _ in 0..20 {
                                let h = MAIN_HWND.load(Ordering::SeqCst);
                                if h != 0 {
                                    unsafe {
                                        windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                                            h as isize, 9, // SW_RESTORE
                                        );
                                        windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(h as isize);
                                    }
                                    dbg_log(&format!("show_main_window() called, hwnd={}", h));
                                    return;
                                }
                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }
                        }

                        // WM_LBUTTONUP = 0x0202, WM_LBUTTONDBLCLK = 0x0203
                        if event == 0x0202 || event == 0x0203 {
                            show_main_window();
                        } else if event == 0x0205 {
                            // WM_RBUTTONUP
                            let mut pt = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
                            GetCursorPos(&mut pt);

                            let menu = CreatePopupMenu();
                            let open_text: Vec<u16> = "Open Controller\0".encode_utf16().collect();
                            let exit_text: Vec<u16> = "Exit App\0".encode_utf16().collect();

                            AppendMenuW(menu, MF_STRING, 1, open_text.as_ptr());
                            AppendMenuW(menu, MF_STRING, 2, exit_text.as_ptr());

                            SetForegroundWindow(hwnd);
                            let cmd = TrackPopupMenu(
                                menu,
                                TPM_LEFTALIGN | TPM_RETURNCMD,
                                pt.x,
                                pt.y,
                                0,
                                hwnd,
                                std::ptr::null(),
                            );
                            DestroyMenu(menu);

                            if cmd == 1 {
                                show_main_window();
                            } else if cmd == 2 {
                                std::process::exit(0);
                            }
                        }
                        return 0;
                    }
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
            }

            let wnd_class = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(tray_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: 0,
                hIcon: 0,
                hCursor: 0,
                hbrBackground: 0,
                lpszMenuName: std::ptr::null(),
                lpszClassName: CLASS_NAME_BYTES.as_ptr(),
            };
            let atom = RegisterClassW(&wnd_class);
            dbg_log(&format!("RegisterClassW returned atom={}", atom));

            // Create message-only window (HWND_MESSAGE = -3)
            let hwnd = CreateWindowExW(
                0,
                CLASS_NAME_BYTES.as_ptr(),
                std::ptr::null(),
                0,
                0,
                0,
                0,
                0,
                -3isize as HWND, // HWND_MESSAGE
                0,
                0,
                std::ptr::null(),
            );
            dbg_log(&format!("CreateWindowExW returned hwnd={}", hwnd));

            if hwnd == 0 {
                dbg_log("ERROR: hwnd is 0, cannot create tray icon");
                return;
            }

            // Load the icon from embedded resource (no external file needed)
            let hinstance = GetModuleHandleW(std::ptr::null());
            let icon_name: Vec<u16> = "keyboard_icon\0".encode_utf16().collect();
            let hicon = LoadImageW(hinstance, icon_name.as_ptr(), IMAGE_ICON, 16, 16, 0) as HICON;
            dbg_log(&format!("LoadImageW (embedded) icon handle={}", hicon));

            // Create the tray icon
            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1;
            nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
            nid.uCallbackMessage = WM_TRAYICON;
            nid.hIcon = hicon;

            let tip: Vec<u16> = "Rust Keyboard LED Controller\0".encode_utf16().collect();
            let len = tip.len().min(nid.szTip.len());
            nid.szTip[..len].copy_from_slice(&tip[..len]);

            let result = Shell_NotifyIconW(NIM_ADD, &nid);
            dbg_log(&format!("Shell_NotifyIconW NIM_ADD result={}", result));

            // Message loop
            let mut msg: MSG = std::mem::zeroed();
            dbg_log("Entering message loop");
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            dbg_log("Message loop exited");

            // Cleanup
            Shell_NotifyIconW(NIM_DELETE, &nid);
        }
    });
}
