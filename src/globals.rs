use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};

pub static TOGGLE_FLAG: AtomicBool = AtomicBool::new(false);
pub static LAST_VK: AtomicU32 = AtomicU32::new(0);

/// Store the main window HWND so tray thread can show/hide it directly via Win32 API
pub static MAIN_HWND: AtomicU64 = AtomicU64::new(0);
/// Cache app directory so PowerShell is only called once
pub static APP_DIR: OnceLock<String> = OnceLock::new();

pub static F8_LAST_TOGGLE: AtomicU64 = AtomicU64::new(0); // debounce for Fn+F8

// ── Custom window messages ──────────────────────────────────

/// Posted to main window when the tray icon is clicked (restore from tray)
pub const WM_RESTORE_WINDOW: u32 = 0x8000 + 10;

/// Posted to main window when Fn+F8 hotkey fires
pub const WM_HOTKEY_TOGGLE: u32 = 0x8000 + 11;

/// Used by the tray message-only window for tray icon notifications
pub const WM_TRAYICON: u32 = 0x8000 + 1;
