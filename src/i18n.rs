use serde::{Deserialize, Serialize};

/// Supported UI languages.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub enum Language {
    #[default]
    English,
    German,
    Turkish,
}

impl Language {
    /// Display name shown in the language selector.
    pub fn display_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::German => "Deutsch",
            Language::Turkish => "Türkçe",
        }
    }

    /// Short code used in config.json.
    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::German => "de",
            Language::Turkish => "tr",
        }
    }

    /// Parse a language code string into a Language enum.
    pub fn from_code(code: &str) -> Language {
        match code {
            "de" => Language::German,
            "tr" => Language::Turkish,
            _ => Language::English,
        }
    }

    /// All available languages, for iteration.
    pub const ALL: [Language; 3] = [Language::English, Language::German, Language::Turkish];
}

/// All translatable UI strings.
#[allow(dead_code)]
pub struct Translations {
    // Brand bar
    pub brand_title: &'static str,
    pub brand_subtitle: &'static str,

    // Main header
    pub header: &'static str,

    // Hardware Effect Mode card
    pub hw_effect_title: &'static str,
    pub hw_effect_warning: &'static str,
    pub effect_label: &'static str,
    pub speed_label: &'static str,
    pub brightness_label: &'static str,
    pub speed_slow: &'static str,
    pub speed_medium: &'static str,
    pub speed_fast: &'static str,

    // Hardware mode names
    pub mode_off: &'static str,
    pub mode_static: &'static str,
    pub mode_breathing: &'static str,
    pub mode_single_flash: &'static str,
    pub mode_double_flash: &'static str,
    pub mode_multicolor: &'static str,
    pub mode_rainbow: &'static str,

    // Software Animation Mode card
    pub sw_anim_title: &'static str,
    pub sw_speed_label: &'static str,
    pub sw_randomize: &'static str,

    // Software mode names
    pub sw_disabled: &'static str,
    pub sw_random_color: &'static str,
    pub sw_smooth_rainbow: &'static str,
    pub sw_smooth_breathing: &'static str,
    pub sw_candle_flicker: &'static str,
    pub sw_ocean_wave: &'static str,
    pub sw_christmas: &'static str,
    pub sw_sunset: &'static str,
    pub sw_pulse: &'static str,
    pub sw_color_cycle: &'static str,
    pub sw_random_chaos: &'static str,

    // Color Configuration card
    pub color_config_title: &'static str,

    // Save button
    pub save_apply: &'static str,

    // Status messages
    pub status_applied: &'static str,
    pub status_error_prefix: &'static str,
    pub status_saved_sw: &'static str,
    pub status_minimized: &'static str,
    pub status_tray: &'static str,
    pub status_hotkey: &'static str,
    pub status_wakeup_ok: &'static str,
    pub status_wakeup_fail: &'static str,

    // Info card
    pub info_title: &'static str,
    pub fn_f8_info: &'static str,

    // Options card
    pub options_title: &'static str,
    pub opt_start_boot: &'static str,
    pub opt_close_tray: &'static str,
    pub opt_start_in_tray: &'static str,

    // Actions card
    pub actions_title: &'static str,
    pub act_force_wakeup: &'static str,
    pub act_refresh: &'static str,
    pub act_exit: &'static str,

    // Language selector
    pub language_label: &'static str,
}

/// Return the translations for the given language.
pub fn translations(lang: Language) -> Translations {
    match lang {
        Language::English => english(),
        Language::German => german(),
        Language::Turkish => turkish(),
    }
}

fn english() -> Translations {
    Translations {
        // Brand bar
        brand_title: "RUST KEYBOARD",
        brand_subtitle: "LED Controller",

        // Main header
        header: "RGB Keyboard Lighting",

        // Hardware Effect Mode card
        hw_effect_title: "Hardware Effect Mode",
        hw_effect_warning: "\u{26A0} Software animation active \u{2014} hardware mode overridden",
        effect_label: "Effect:",
        speed_label: "Speed:",
        brightness_label: "Brightness:",
        speed_slow: "Slow",
        speed_medium: "Medium",
        speed_fast: "Fast",

        // Hardware mode names
        mode_off: "Off",
        mode_static: "Static Single Color",
        mode_breathing: "Breathing Wave",
        mode_single_flash: "Single Flashing",
        mode_double_flash: "Double Flashing",
        mode_multicolor: "Multi-color Shift",
        mode_rainbow: "Rainbow",

        // Software Animation Mode card
        sw_anim_title: "Software Animation Mode",
        sw_speed_label: "Speed:",
        sw_randomize: "\u{1F3B2} Randomize speed",

        // Software mode names
        sw_disabled: "\u{23F8}  Disabled (Hardware Mode)",
        sw_random_color: "\u{1F3B2}  Random Color",
        sw_smooth_rainbow: "\u{1F308}  Smooth Rainbow",
        sw_smooth_breathing: "\u{1FAC1}  Smooth Breathing",
        sw_candle_flicker: "\u{1F525}  Candle Flicker",
        sw_ocean_wave: "\u{1F30A}  Ocean Wave",
        sw_christmas: "\u{1F384}  Christmas",
        sw_sunset: "\u{1F305}  Sunset",
        sw_pulse: "\u{1F3B5}  Pulse",
        sw_color_cycle: "\u{1F49C}  Color Cycle",
        sw_random_chaos: "\u{1F500}  Random Chaos",

        // Color Configuration card
        color_config_title: "Color Configuration",

        // Save button
        save_apply: "\u{1F4BE}  Save & Apply to Keyboard",

        // Status messages
        status_applied: "Applied & saved!",
        status_error_prefix: "Error:",
        status_saved_sw: "Saved! Software animation active.",
        status_minimized: "Started in System Tray",
        status_tray: "Minimized to System Tray",
        status_hotkey: "Keyboard toggled via Hotkey!",
        status_wakeup_ok: "Wake up sent!",
        status_wakeup_fail: "Wake up failed!",

        // Info card
        info_title: "Info",
        fn_f8_info: "Fn+F8: Cycle Hardware Modes",

        // Options card
        options_title: "Options",
        opt_start_boot: "\u{1F680} Start on Boot",
        opt_close_tray: "\u{1F4E5} Close to Tray",
        opt_start_in_tray: "\u{1F4E5} Start in Tray",

        // Actions card
        actions_title: "Actions",
        act_force_wakeup: "\u{26A1}  Force Wake Up",
        act_refresh: "\u{1F504}  Refresh",
        act_exit: "\u{274C}  Exit Application",

        // Language selector
        language_label: "Language:",
    }
}

fn german() -> Translations {
    Translations {
        // Brand bar
        brand_title: "RUST KEYBOARD",
        brand_subtitle: "LED-Steuerung",

        // Main header
        header: "RGB-Tastaturbeleuchtung",

        // Hardware Effect Mode card
        hw_effect_title: "Hardware-Effektmodus",
        hw_effect_warning: "\u{26A0} Software-Animation aktiv \u{2014} Hardware-Modus überschrieben",
        effect_label: "Effekt:",
        speed_label: "Geschwindigkeit:",
        brightness_label: "Helligkeit:",
        speed_slow: "Langsam",
        speed_medium: "Mittel",
        speed_fast: "Schnell",

        // Hardware mode names
        mode_off: "Aus",
        mode_static: "Statische Einzelfarbe",
        mode_breathing: "Atmende Welle",
        mode_single_flash: "Einzelnes Blinken",
        mode_double_flash: "Doppeltes Blinken",
        mode_multicolor: "Mehrfarben-Wechsel",
        mode_rainbow: "Regenbogen",

        // Software Animation Mode card
        sw_anim_title: "Software-Animationsmodus",
        sw_speed_label: "Geschwindigkeit:",
        sw_randomize: "\u{1F3B2} Zufällige Geschwindigkeit",

        // Software mode names
        sw_disabled: "\u{23F8}  Deaktiviert (Hardware-Modus)",
        sw_random_color: "\u{1F3B2}  Zufällige Farbe",
        sw_smooth_rainbow: "\u{1F308}  Sanfter Regenbogen",
        sw_smooth_breathing: "\u{1FAC1}  Sanftes Atmen",
        sw_candle_flicker: "\u{1F525}  Kerzenschein",
        sw_ocean_wave: "\u{1F30A}  Ozeanwelle",
        sw_christmas: "\u{1F384}  Weihnachten",
        sw_sunset: "\u{1F305}  Sonnenuntergang",
        sw_pulse: "\u{1F3B5}  Puls",
        sw_color_cycle: "\u{1F49C}  Farbzyklus",
        sw_random_chaos: "\u{1F500}  Zufalls-Chaos",

        // Color Configuration card
        color_config_title: "Farbkonfiguration",

        // Save button
        save_apply: "\u{1F4BE}  Speichern & Anwenden",

        // Status messages
        status_applied: "Angewendet & gespeichert!",
        status_error_prefix: "Fehler:",
        status_saved_sw: "Gespeichert! Software-Animation aktiv.",
        status_minimized: "Im System-Tray gestartet",
        status_tray: "In den System-Tray minimiert",
        status_hotkey: "Tastatur per Hotkey umgeschaltet!",
        status_wakeup_ok: "Aufwecken gesendet!",
        status_wakeup_fail: "Aufwecken fehlgeschlagen!",

        // Info card
        info_title: "Info",
        fn_f8_info: "Fn+F8: Hardware-Modi durchschalten",

        // Options card
        options_title: "Optionen",
        opt_start_boot: "\u{1F680} Autostart",
        opt_close_tray: "\u{1F4E5} In Tray schließen",
        opt_start_in_tray: "\u{1F4E5} Im System-Tray starten",

        // Actions card
        actions_title: "Aktionen",
        act_force_wakeup: "\u{26A1}  Aufwecken erzwingen",
        act_refresh: "\u{1F504}  Aktualisieren",
        act_exit: "\u{274C}  Anwendung beenden",

        // Language selector
        language_label: "Sprache:",
    }
}

fn turkish() -> Translations {
    Translations {
        // Brand bar
        brand_title: "RUST KEYBOARD",
        brand_subtitle: "LED Kontrol",

        // Main header
        header: "RGB Klavye Aydınlatması",

        // Hardware Effect Mode card
        hw_effect_title: "Donanım Efekt Modu",
        hw_effect_warning: "\u{26A0} Yazılım animasyonu aktif \u{2014} donanım modu geçersiz kılındı",
        effect_label: "Efekt:",
        speed_label: "Hız:",
        brightness_label: "Parlaklık:",
        speed_slow: "Yavaş",
        speed_medium: "Orta",
        speed_fast: "Hızlı",

        // Hardware mode names
        mode_off: "Kapalı",
        mode_static: "Sabit Tek Renk",
        mode_breathing: "Nefes Alan Dalga",
        mode_single_flash: "Tekli Yanıp Sönme",
        mode_double_flash: "Çiftli Yanıp Sönme",
        mode_multicolor: "Çok Renkli Geçiş",
        mode_rainbow: "Gökkuşağı",

        // Software Animation Mode card
        sw_anim_title: "Yazılım Animasyon Modu",
        sw_speed_label: "Hız:",
        sw_randomize: "\u{1F3B2} Rastgele hız",

        // Software mode names
        sw_disabled: "\u{23F8}  Devre Dışı (Donanım Modu)",
        sw_random_color: "\u{1F3B2}  Rastgele Renk",
        sw_smooth_rainbow: "\u{1F308}  Yumuşak Gökkuşağı",
        sw_smooth_breathing: "\u{1FAC1}  Yumuşak Nefes",
        sw_candle_flicker: "\u{1F525}  Mum Titremesi",
        sw_ocean_wave: "\u{1F30A}  Okyanus Dalgası",
        sw_christmas: "\u{1F384}  Noel",
        sw_sunset: "\u{1F305}  Gün Batımı",
        sw_pulse: "\u{1F3B5}  Nabız",
        sw_color_cycle: "\u{1F49C}  Renk Döngüsü",
        sw_random_chaos: "\u{1F500}  Rastgele Kaos",

        // Color Configuration card
        color_config_title: "Renk Yapılandırması",

        // Save button
        save_apply: "\u{1F4BE}  Kaydet ve Uygula",

        // Status messages
        status_applied: "Uygulandı ve kaydedildi!",
        status_error_prefix: "Hata:",
        status_saved_sw: "Kaydedildi! Yazılım animasyonu aktif.",
        status_minimized: "Sistem tepsisinde başlatıldı",
        status_tray: "Sistem tepsisine küçültüldü",
        status_hotkey: "Klavye kısayol tuşuyla değiştirildi!",
        status_wakeup_ok: "Uyandırma gönderildi!",
        status_wakeup_fail: "Uyandırma başarısız!",

        // Info card
        info_title: "Bilgi",
        fn_f8_info: "Fn+F8: Donanım Modları Arasında Geçiş",

        // Options card
        options_title: "Seçenekler",
        opt_start_boot: "\u{1F680} Başlangıçta Çalıştır",
        opt_close_tray: "\u{1F4E5} Tepsiye Kapat",
        opt_start_in_tray: "\u{1F4E5} Sistem Tepsisinde Başlat",

        // Actions card
        actions_title: "İşlemler",
        act_force_wakeup: "\u{26A1}  Zorla Uyandır",
        act_refresh: "\u{1F504}  Yenile",
        act_exit: "\u{274C}  Uygulamadan Çık",

        // Language selector
        language_label: "Dil:",
    }
}

/// Helper: get the translated software mode name from a Translations struct.
#[allow(dead_code)]
pub fn software_mode_name_translated(t: &Translations, mode: u8) -> &'static str {
    match mode {
        0 => t.sw_disabled,
        1 => t.sw_random_color,
        2 => t.sw_smooth_rainbow,
        3 => t.sw_smooth_breathing,
        4 => t.sw_candle_flicker,
        5 => t.sw_ocean_wave,
        6 => t.sw_christmas,
        7 => t.sw_sunset,
        8 => t.sw_pulse,
        9 => t.sw_color_cycle,
        10 => t.sw_random_chaos,
        _ => "Unknown",
    }
}
