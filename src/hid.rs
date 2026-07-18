use crate::config::{KeyboardConfig, save_config};

// ── Helpers ────────────────────────────────────────────────

/// MSI keyboard PIDs to try when opening the device
const MSI_PIDS: [u16; 3] = [5474, 5475, 5476];

/// Try to open an MSI keyboard device through any known PID
pub fn open_msi_keyboard(hid: &hidapi::HidApi) -> Option<hidapi::HidDevice> {
    for &pid in &MSI_PIDS {
        if let Ok(device) = hid.open(0x1462, pid) {
            return Some(device);
        }
    }
    None
}

// ── Public API ─────────────────────────────────────────────

pub fn recover_keyboard() -> bool {
    if let Ok(hid) = hidapi::HidApi::new() {
        // Artery Bootloader: VID = 0x2E3C, PID = 0xAF01
        if let Ok(device) = hid.open(0x2E3C, 0xAF01) {
            let mut buf = [0u8; 65];
            buf[0] = 1; // Report ID
            buf[1] = 0x5A; // Sync
            buf[2] = 0xA6; // Jump to Application
            if device.write(&buf).is_ok() {
                return true;
            }
        }
    }
    false
}

pub fn apply_lighting_config(hid: &hidapi::HidApi, config: &KeyboardConfig) -> Result<(), String> {
    let device = open_msi_keyboard(hid).ok_or("MSI keyboard not found".to_string())?;

    let mut buf = [0u8; 64];
    buf[0] = 2; // Report ID
    buf[1] = 0;
    buf[2] = config.mode;
    buf[3] = config.speed;
    buf[4] = config.brightness;
    buf[5] = 7; // Colors count

    for (i, color) in config.colors.iter().enumerate().take(7) {
        let offset = i * 3 + 6;
        if offset + 2 < buf.len() {
            buf[offset] = color[0];
            buf[offset + 1] = color[1];
            buf[offset + 2] = color[2];
        }
    }

    device.send_feature_report(&buf).map_err(|e| e.to_string())
}

/// Save config to disk and apply hardware lighting in one call
pub fn save_and_apply(config: &KeyboardConfig) -> Result<(), String> {
    let _ = save_config(config);
    if let Ok(hid) = hidapi::HidApi::new() {
        apply_lighting_config(&hid, config)
    } else {
        Err("HID API not available".to_string())
    }
}

pub fn get_keyboard_status() -> String {
    if let Ok(hid) = hidapi::HidApi::new() {
        let has_bootloader = hid
            .device_list()
            .any(|d| d.vendor_id() == 0x2E3C && d.product_id() == 0xAF01);
        let has_normal = hid
            .device_list()
            .any(|d| d.vendor_id() == 0x1462 && MSI_PIDS.contains(&d.product_id()));

        if has_bootloader {
            "Connected (Bootloader Mode)".to_string()
        } else if has_normal {
            "Connected (Normal Operating Mode)".to_string()
        } else {
            "Not Found".to_string()
        }
    } else {
        "Error accessing HID API".to_string()
    }
}
