#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod globals;
mod hid;
mod hook;
mod i18n;
mod startup;
mod tray;
mod window;

fn main() {
    hook::start_keyboard_hook();
    tray::start_tray_icon_thread();

    if let Err(e) = window::AppWindow::create() {
        let title: Vec<u16> = "Error\0".encode_utf16().collect();
        let msg: Vec<u16> = format!("Failed to create window: {}\0", e)
            .encode_utf16()
            .collect();
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
                0, // HWND = 0
                msg.as_ptr(),
                title.as_ptr(),
                windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
            );
        }
    }
}
