use std::os::windows::process::CommandExt;
use std::process::Command;

/// Create a PowerShell command with CREATE_NO_WINDOW to prevent console flash
fn powershell(script: &str) -> std::io::Result<std::process::Output> {
    Command::new("powershell.exe")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .args(["-NoProfile", "-Command", script])
        .output()
}

/// Check if "Start on Boot" is enabled in Windows Registry
pub fn is_startup_enabled() -> bool {
    fn check_registry(hive: &str) -> bool {
        let cmd = format!(
            "Get-ItemProperty -Path '{0}:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'RustKeyboardGUI' -ErrorAction SilentlyContinue | Select-Object -ExpandProperty RustKeyboardGUI -ErrorAction SilentlyContinue",
            hive
        );
        if let Ok(out) = powershell(&cmd) {
            !String::from_utf8_lossy(&out.stdout).trim().is_empty()
        } else {
            false
        }
    }

    check_registry("HKLM") || check_registry("HKCU")
}

/// Enable or disable "Start on Boot" via Windows Registry
pub fn set_startup(enable: bool) -> std::io::Result<()> {
    if enable {
        let current_exe = std::env::current_exe()?;
        let exe_path = current_exe.to_string_lossy().replace('\'', "''");

        // Try HKLM first (all users, requires admin)
        let cmd_hklm = format!(
            "Set-ItemProperty -Path 'HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'RustKeyboardGUI' -Value '\"{}\"'",
            exe_path
        );
        let output = powershell(&cmd_hklm)?;
        if !output.status.success() {
            // Fallback to HKCU if HKLM fails (no admin rights)
            let cmd_hkcu = format!(
                "Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'RustKeyboardGUI' -Value '\"{}\"'",
                exe_path
            );
            let _ = powershell(&cmd_hkcu)?;
        }
    } else {
        let _ = powershell(
            "Remove-ItemProperty -Path 'HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'RustKeyboardGUI' -ErrorAction SilentlyContinue",
        );
        let _ = powershell(
            "Remove-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'RustKeyboardGUI' -ErrorAction SilentlyContinue",
        );
    }
    Ok(())
}
