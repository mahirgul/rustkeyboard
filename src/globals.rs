use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};

pub static TOGGLE_FLAG: AtomicBool = AtomicBool::new(false);
pub static LAST_VK: AtomicU32 = AtomicU32::new(0);

pub static EGUI_CTX: std::sync::Mutex<Option<eframe::egui::Context>> = std::sync::Mutex::new(None);
/// Store the main window HWND so tray thread can show/hide it directly via Win32 API
pub static MAIN_HWND: AtomicU64 = AtomicU64::new(0);
/// Cache app directory so PowerShell is only called once
pub static APP_DIR: OnceLock<String> = OnceLock::new();

pub static F8_LAST_TOGGLE: AtomicU64 = AtomicU64::new(0); // debounce for Fn+F8

pub const WM_TRAYICON: u32 = 0x8000 + 1; // WM_USER + 1
