use windows_sys::Win32::System::Registry::*;

fn wstr(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Check if "Start on Boot" is enabled in Windows Registry
pub fn is_startup_enabled() -> bool {
    let subkey = wstr(r"Software\Microsoft\Windows\CurrentVersion\Run");
    let value_name = wstr("RustKeyboardGUI");

    unsafe {
        for &hive in &[HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
            let mut hkey = 0;
            if RegOpenKeyExW(hive, subkey.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
                let mut data_type = 0;
                let mut cb_data = 0;
                let status = RegQueryValueExW(
                    hkey,
                    value_name.as_ptr(),
                    std::ptr::null_mut(),
                    &mut data_type,
                    std::ptr::null_mut(),
                    &mut cb_data,
                );
                RegCloseKey(hkey);
                if status == 0 {
                    return true;
                }
            }
        }
    }
    false
}

/// Enable or disable "Start on Boot" via Windows Registry
pub fn set_startup(enable: bool) -> std::io::Result<()> {
    let subkey = wstr(r"Software\Microsoft\Windows\CurrentVersion\Run");
    let value_name = wstr("RustKeyboardGUI");

    unsafe {
        if enable {
            let current_exe = std::env::current_exe()?;
            let exe_path = current_exe.to_string_lossy().to_string();
            let formatted_path = format!("\"{}\"", exe_path);
            let path_w = wstr(&formatted_path);
            let cb_data = (path_w.len() * 2) as u32;

            // Try HKLM first (requires admin/write access)
            let mut hkey = 0;
            let mut status =
                RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_WRITE, &mut hkey);
            if status == 0 {
                status = RegSetValueExW(
                    hkey,
                    value_name.as_ptr(),
                    0,
                    REG_SZ,
                    path_w.as_ptr() as *const u8,
                    cb_data,
                );
                RegCloseKey(hkey);
            }

            if status != 0 {
                // Fallback to HKCU
                let mut hkey = 0;
                if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_WRITE, &mut hkey) == 0 {
                    let _ = RegSetValueExW(
                        hkey,
                        value_name.as_ptr(),
                        0,
                        REG_SZ,
                        path_w.as_ptr() as *const u8,
                        cb_data,
                    );
                    RegCloseKey(hkey);
                }
            }
        } else {
            // Delete from both HKLM and HKCU silently
            for &hive in &[HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
                let mut hkey = 0;
                if RegOpenKeyExW(hive, subkey.as_ptr(), 0, KEY_WRITE, &mut hkey) == 0 {
                    let _ = RegDeleteValueW(hkey, value_name.as_ptr());
                    RegCloseKey(hkey);
                }
            }
        }
    }
    Ok(())
}
