use serde::{Deserialize, Serialize};
use std::fs;

use crate::globals::APP_DIR;
use crate::i18n::Language;

fn default_close_to_tray() -> bool {
    true
}

fn default_start_in_tray() -> bool {
    false
}

fn default_software_mode() -> u8 {
    0
}

fn default_sw_speed_ms() -> u32 {
    2000
}

fn default_sw_random_speed() -> bool {
    false
}

fn default_language() -> String {
    "en".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyboardConfig {
    pub mode: u8,
    pub speed: u8,
    pub brightness: u8,
    pub colors: Vec<[u8; 3]>,

    #[serde(default = "default_close_to_tray")]
    pub close_to_tray: bool,

    #[serde(
        default = "default_start_in_tray",
        rename = "start_in_tray",
        alias = "start_minimized"
    )]
    pub start_in_tray: bool,

    #[serde(default = "default_software_mode")]
    pub software_mode: u8,

    #[serde(default = "default_sw_speed_ms")]
    pub sw_speed_ms: u32,

    #[serde(default = "default_sw_random_speed")]
    pub sw_random_speed: bool,

    #[serde(default = "default_language")]
    pub language: String,
}

impl KeyboardConfig {
    /// Return the first user-defined color, or white if no colors configured
    #[allow(dead_code)]
    pub fn first_color(&self) -> [u8; 3] {
        self.colors.first().copied().unwrap_or([255, 255, 255])
    }

    /// Ensure the colors vector has exactly 7 entries (pad/trim as needed)
    pub fn normalize(&mut self) {
        while self.colors.len() < 7 {
            self.colors.push([255, 255, 255]);
        }
        self.colors.truncate(7);
    }

    /// Convert the stored language string to a Language enum.
    pub fn language_enum(&self) -> Language {
        Language::from_code(&self.language)
    }

    /// Set the language from a Language enum value.
    pub fn set_language(&mut self, lang: Language) {
        self.language = lang.code().to_string();
    }
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            mode: 1,        // Static
            speed: 1,       // Medium
            brightness: 10, // Max
            colors: vec![
                [255, 255, 255], // Color 0: White
                [255, 0, 0],     // Color 1: Red
                [255, 165, 0],   // Color 2: Orange
                [255, 255, 0],   // Color 3: Yellow
                [0, 255, 0],     // Color 4: Green
                [0, 255, 255],   // Color 5: Cyan
                [255, 0, 255],   // Color 6: Purple
            ],
            close_to_tray: true,
            start_in_tray: false,
            software_mode: 0,
            sw_speed_ms: 2000,
            sw_random_speed: false,
            language: default_language(),
        }
    }
}

/// Resolve the application directory (same directory as the exe)
pub fn get_app_dir() -> String {
    APP_DIR
        .get_or_init(|| {
            if let Ok(exe) = std::env::current_exe()
                && let Some(parent) = exe.parent()
            {
                return parent.to_string_lossy().to_string();
            }
            "C:\\rustkeyboard".to_string()
        })
        .clone()
}

pub fn get_config_path() -> String {
    format!("{}\\config.json", get_app_dir())
}

pub fn load_config() -> Option<KeyboardConfig> {
    let path = get_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        match serde_json::from_str::<KeyboardConfig>(&content) {
            Ok(mut config) => {
                config.normalize(); // ensure 7 colors after load
                Some(config)
            }
            Err(_) => {
                // Corrupt config — overwrite with defaults
                let default_config = KeyboardConfig::default();
                let _ = save_config(&default_config);
                Some(default_config)
            }
        }
    } else {
        // Config file doesn't exist — auto-create with defaults
        let default_config = KeyboardConfig::default();
        let _ = save_config(&default_config);
        Some(default_config)
    }
}

pub fn save_config(config: &KeyboardConfig) -> std::io::Result<()> {
    let dir = get_app_dir();
    let _ = fs::create_dir_all(&dir);
    let content = serde_json::to_string_pretty(config)?;
    fs::write(get_config_path(), content)
}

pub fn dbg_log(msg: &str) {
    use std::io::Write;
    let log_path = format!("{}\\tray_debug.log", get_app_dir());
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(f, "[{:?}] {}", std::time::SystemTime::now(), msg);
    }
}
