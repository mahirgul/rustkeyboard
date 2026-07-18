use std::sync::atomic::Ordering;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::config::{KeyboardConfig, load_config, save_config};
use crate::globals::{MAIN_HWND, WM_HOTKEY_TOGGLE, WM_RESTORE_WINDOW};
use crate::hid::{apply_lighting_config, get_keyboard_status, recover_keyboard, save_and_apply};
use crate::i18n::{self, Language};
use crate::startup::{is_startup_enabled, set_startup};

// ── Custom FFI for ChooseColor ─────────────────────────────
#[repr(C)]
#[allow(non_snake_case)]
#[allow(clippy::upper_case_acronyms)]
struct CHOOSECOLORW {
    lStructSize: u32,
    hwndOwner: HWND,
    hInstance: HINSTANCE,
    rgbResult: COLORREF,
    lpCustColors: *mut COLORREF,
    Flags: u32,
    lCustData: LPARAM,
    lpfnHook: usize,
    lpTemplateName: *const u16,
}
unsafe extern "system" {
    fn ChooseColorW(lpcc: *mut CHOOSECOLORW) -> BOOL;
}
const CC_RGBINIT: u32 = 0x0001;
const CC_FULLOPEN: u32 = 0x0002;
const TBM_GETPOS: u32 = 0x0400;
const SS_LEFT_STYLE: u32 = 0x0000;

// ── Colors ─────────────────────────────────────────────────
const COLOR_HEADER_BG: u32 = 0x001A2E; // Deep navy

// ── Control IDs ────────────────────────────────────────────
const IDC_EFFECT: u16 = 101;
const IDC_SPEED: u16 = 102;
const IDC_BRIGHTNESS: u16 = 103;
const IDC_COLOR_BASE: u16 = 200;
const IDC_LANGUAGE: u16 = 300;
const IDC_AUTOSTART: u16 = 401;
const IDC_CLOSE_TRAY: u16 = 402;
const IDC_START_TRAY: u16 = 403;
const IDC_SAVE: u16 = 501;
const IDC_WAKEUP: u16 = 502;
const IDC_REFRESH: u16 = 503;
const IDC_EXIT: u16 = 504;
const IDC_HEADER_PANEL: u16 = 601;

// ── Window class name ──────────────────────────────────────
const WND_CLASS: &[u16] = &[
    b'R' as u16,
    b'K' as u16,
    b'G' as u16,
    b'u' as u16,
    b'i' as u16,
    0,
];

fn wstr(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub struct AppWindow {
    config: KeyboardConfig,
    status: String,
    autostart: bool,
    is_visible: bool,
    hwnd: HWND,
    // Fonts
    hfont_title: HFONT,
    hfont_section: HFONT,
    hfont_normal: HFONT,
    hfont_small: HFONT,
    // Brushes
    hbrush_header: HBRUSH,
    hbrush_white: HBRUSH,
    // Child controls
    hwnd_effect: HWND,
    hwnd_speed: HWND,
    hwnd_brightness: HWND,
    hwnd_brightness_label: HWND,
    hwnd_color_btns: [HWND; 7],
    hwnd_language: HWND,
    hwnd_autostart: HWND,
    hwnd_close_tray: HWND,
    hwnd_start_tray: HWND,
    hwnd_status: HWND,
    hwnd_header_panel: HWND,
    hwnd_header_title: HWND,
    hwnd_header_sub: HWND,
    hwnd_brand_line: HWND,
    color_brushes: [HBRUSH; 7],
}

impl AppWindow {
    pub fn create() -> Result<(), String> {
        unsafe {
            let mut icc: INITCOMMONCONTROLSEX = std::mem::zeroed();
            icc.dwSize = std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32;
            icc.dwICC = ICC_STANDARD_CLASSES | ICC_BAR_CLASSES;
            InitCommonControlsEx(&icc);
        }

        unsafe {
            let hinstance = GetModuleHandleW(std::ptr::null());
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: LoadIconW(hinstance, wstr("keyboard_icon").as_ptr()),
                hCursor: LoadCursorW(0, IDC_ARROW),
                hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
                lpszMenuName: std::ptr::null(),
                lpszClassName: WND_CLASS.as_ptr(),
            };
            if RegisterClassW(&wc) == 0 {
                return Err("Failed to register window class".into());
            }
        }

        let config = load_config().unwrap_or_default();
        let start_in_tray = config.start_in_tray;
        let dw_style: u32 = if start_in_tray { 0 } else { WS_VISIBLE }
            | WS_OVERLAPPEDWINDOW & !(WS_MAXIMIZEBOX | WS_THICKFRAME);

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                WND_CLASS.as_ptr(),
                wstr("Rust Keyboard LED Controller").as_ptr(),
                dw_style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                420,
                660,
                0,
                0,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        };
        if hwnd == 0 {
            return Err("Failed to create window".into());
        }
        MAIN_HWND.store(hwnd as u64, Ordering::SeqCst);

        if start_in_tray {
            unsafe {
                let mut ex = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
                ex &= !WS_EX_APPWINDOW;
                ex |= WS_EX_TOOLWINDOW;
                SetWindowLongW(hwnd, GWL_EXSTYLE, ex as i32);
            }
        }

        let mut msg: MSG = unsafe { std::mem::zeroed() };
        loop {
            let ret = unsafe { GetMessageW(&mut msg, 0, 0, 0) };
            if ret <= 0 {
                break;
            }
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        Ok(())
    }

    fn init(&mut self) {
        // Create brushes
        unsafe {
            self.hbrush_header = CreateSolidBrush(COLOR_HEADER_BG);
            self.hbrush_white = CreateSolidBrush(0xFFFFFF);
        }

        // Create fonts (fixed pixel sizes, no DPI scaling)
        unsafe {
            self.hfont_title = CreateFontW(
                -26,
                0,
                0,
                0,
                FW_BOLD as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET as u32,
                OUT_DEFAULT_PRECIS as u32,
                CLIP_DEFAULT_PRECIS as u32,
                CLEARTYPE_QUALITY as u32,
                (DEFAULT_PITCH | FF_DONTCARE) as u32,
                wstr("Segoe UI").as_ptr(),
            );
            self.hfont_section = CreateFontW(
                -15,
                0,
                0,
                0,
                FW_BOLD as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET as u32,
                OUT_DEFAULT_PRECIS as u32,
                CLIP_DEFAULT_PRECIS as u32,
                CLEARTYPE_QUALITY as u32,
                (DEFAULT_PITCH | FF_DONTCARE) as u32,
                wstr("Segoe UI").as_ptr(),
            );
            self.hfont_normal = CreateFontW(
                -13,
                0,
                0,
                0,
                FW_NORMAL as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET as u32,
                OUT_DEFAULT_PRECIS as u32,
                CLIP_DEFAULT_PRECIS as u32,
                CLEARTYPE_QUALITY as u32,
                (DEFAULT_PITCH | FF_DONTCARE) as u32,
                wstr("Segoe UI").as_ptr(),
            );
            self.hfont_small = CreateFontW(
                -11,
                0,
                0,
                0,
                FW_NORMAL as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET as u32,
                OUT_DEFAULT_PRECIS as u32,
                CLIP_DEFAULT_PRECIS as u32,
                CLEARTYPE_QUALITY as u32,
                (DEFAULT_PITCH | FF_DONTCARE) as u32,
                wstr("Segoe UI").as_ptr(),
            );
        }

        let t = i18n::translations(self.config.language_enum());
        self.create_controls(&t);

        for i in 0..7 {
            self.update_color_button(i);
        }

        // Initial keyboard setup
        let did_wakeup = recover_keyboard();
        if did_wakeup {
            for attempt in 0..10 {
                std::thread::sleep(std::time::Duration::from_millis(if attempt == 0 {
                    300
                } else {
                    500
                }));
                if let Ok(hid) = hidapi::HidApi::new()
                    && apply_lighting_config(&hid, &self.config).is_ok()
                {
                    break;
                }
            }
        } else if let Ok(hid) = hidapi::HidApi::new() {
            let _ = apply_lighting_config(&hid, &self.config);
        }

        self.status = get_keyboard_status();
        self.autostart = is_startup_enabled();
        self.update_checkboxes();
        self.update_status_text();
        self.is_visible = !self.config.start_in_tray;
    }

    // ══════════════════════════════════════════════════════════
    //  LAYOUT  (420 x 660)
    //
    //  ┌──────────────────────────────────────────────┐ y=0
    //  │  ██ HEADER PANEL (navy bg, white text)    ██ │ y=0..60
    //  │  ██  RUST KEYBOARD LED CONTROLLER         ██ │ y=5
    //  │  ██  ─── accent line ───                  ██ │ y=36
    //  │  ██  v2.0 • MSI RGB Keyboard              ██ │ y=44
    //  ├──────────────────────────────────────────────┤ y=62
    //  │  Status: Connected (Normal)                  │ y=68
    //  │  ┌─ Hardware Effect Mode ──────────────────┐ │ y=92
    //  │  │ Effect:  [ComboBox               ▼]     │ │
    //  │  │ Speed:   [ComboBox               ▼]     │ │
    //  │  │ Bright:  [====○====]  High              │ │
    //  │  └─────────────────────────────────────────┘ │ y=212
    //  │  ┌─ Color Configuration ───────────────────┐ │ y=220
    //  │  │  [#1][#2][#3][#4][#5][#6][#7]          │ │
    //  │  └─────────────────────────────────────────┘ │ y=276
    //  │  Language: [ComboBox               ▼]       │ y=288
    //  │  ┌─ Options ───────────────────────────────┐ │ y=320
    //  │  │ ☐ Start on Boot                         │ │
    //  │  │ ☐ Close to Tray                         │ │
    //  │  └─────────────────────────────────────────┘ │ y=390
    //  │  ┌─ Actions ───────────────────────────────┐ │ y=400
    //  │  │ [💾 Save & Apply]     (full width)      │ │
    //  │  │ [⚡ Force Wake Up]    (full width)      │ │
    //  │  │ [🔄 Refresh Status]   (full width)      │ │
    //  │  │ ☐ Start in Tray                         │ │
    //  │  │ [❌ Exit Application] (full width, red)  │ │
    //  │  └─────────────────────────────────────────┘ │ y=600
    //  │  Fn+F8: Cycle hardware modes                │ y=612
    //  └──────────────────────────────────────────────┘ y=660
    // ══════════════════════════════════════════════════════

    fn create_controls(&mut self, t: &crate::i18n::Translations) {
        let h = self.hwnd;
        let ww: i32 = 420; // window client width

        // ── Header panel (navy background) ──────────────
        self.hwnd_header_panel = unsafe {
            CreateWindowExW(
                0,
                wstr("STATIC").as_ptr(),
                std::ptr::null(),
                WS_CHILD | WS_VISIBLE | SS_LEFT_STYLE,
                0,
                0,
                ww,
                64,
                h,
                IDC_HEADER_PANEL as isize as _,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        };

        // Brand title
        self.hwnd_header_title =
            self.create_label_wh(t.brand_title, 16, 6, ww - 32, 30, self.hfont_title);
        // Accent line
        self.hwnd_brand_line = self.create_label_wh(" ", 16, 36, ww - 32, 4, 0);
        // Subtitle
        self.hwnd_header_sub =
            self.create_label_wh(t.brand_subtitle, 16, 42, ww - 32, 18, self.hfont_small);

        // ── Status ──────────────────────────────────────
        self.hwnd_status = self.create_label_wh("", 14, 70, ww - 28, 20, self.hfont_normal);

        // ── Hardware Effect Mode ────────────────────────
        let gy: i32 = 96;
        self.create_group(t.hw_effect_title, 8, gy, ww - 16, 118, self.hfont_section);
        self.create_label_wh(t.effect_label, 18, gy + 24, 70, 22, 0);
        self.hwnd_effect = self.create_combo(IDC_EFFECT, 90, gy + 22, 200, 260);
        self.populate_effect_combo(t);

        self.create_label_wh(t.speed_label, 18, gy + 52, 70, 22, 0);
        self.hwnd_speed = self.create_combo(IDC_SPEED, 90, gy + 50, 200, 260);
        self.populate_speed_combo(t);

        self.create_label_wh(t.brightness_label, 18, gy + 80, 70, 22, 0);
        self.hwnd_brightness = unsafe {
            CreateWindowExW(
                0,
                TRACKBAR_CLASSW,
                std::ptr::null(),
                WS_CHILD | WS_VISIBLE | TBS_AUTOTICKS,
                90,
                gy + 78,
                200,
                28,
                h,
                IDC_BRIGHTNESS as isize as _,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        };
        unsafe {
            SendMessageW(self.hwnd_brightness, TBM_SETRANGE, 1, (3u32 << 16) as isize);
            SendMessageW(self.hwnd_brightness, TBM_SETTICFREQ, 1, 0);
            let bp = (3i32 - self.config.brightness.min(3) as i32) as isize;
            SendMessageW(self.hwnd_brightness, TBM_SETPOS, 1, bp);
        }
        self.hwnd_brightness_label =
            self.create_label_wh("", 298, gy + 80, 100, 22, self.hfont_small);

        // ── Color Configuration ─────────────────────────
        let cy: i32 = gy + 126;
        self.create_group(t.color_config_title, 8, cy, ww - 16, 58, self.hfont_section);
        for i in 0..7i32 {
            let x = 22 + i * 52;
            self.hwnd_color_btns[i as usize] = self.create_color_button(
                &format!("#{}", i + 1),
                IDC_COLOR_BASE + i as u16,
                x,
                cy + 22,
                48,
                26,
            );
        }

        // ── Language ─────────────────────────────────────
        let ly: i32 = cy + 66;
        self.create_label_wh(t.language_label, 18, ly + 2, 80, 22, 0);
        self.hwnd_language = self.create_combo(IDC_LANGUAGE, 100, ly, 150, 260);
        self.populate_language_combo(t);

        // ── Options ─────────────────────────────────────
        let oy: i32 = ly + 34;
        self.create_group(t.options_title, 8, oy, ww - 16, 62, self.hfont_section);
        self.hwnd_autostart =
            self.create_checkbox(t.opt_start_boot, IDC_AUTOSTART, 18, oy + 22, ww - 40, 20);
        self.hwnd_close_tray =
            self.create_checkbox(t.opt_close_tray, IDC_CLOSE_TRAY, 18, oy + 44, ww - 40, 20);

        // ── Actions ─────────────────────────────────────
        let ay: i32 = oy + 72;
        self.create_group(t.actions_title, 8, ay, ww - 16, 178, self.hfont_section);

        // Stacked buttons - full width
        let bx: i32 = 18;
        let bw: i32 = ww - 44;

        self.create_button(t.save_apply, IDC_SAVE, bx, ay + 22, bw, 30);
        self.create_button(t.act_force_wakeup, IDC_WAKEUP, bx, ay + 56, bw, 30);
        self.create_button(t.act_refresh, IDC_REFRESH, bx, ay + 90, bw, 30);

        self.hwnd_start_tray =
            self.create_checkbox(t.opt_start_in_tray, IDC_START_TRAY, bx, ay + 128, bw, 20);
        self.create_button(t.act_exit, IDC_EXIT, bx, ay + 152, bw, 30);

        // ── Info ─────────────────────────────────────────
        let iy: i32 = ay + 186;
        self.create_label_wh(
            &format!("  {}", t.fn_f8_info),
            14,
            iy,
            ww - 28,
            16,
            self.hfont_small,
        );

        self.update_controls_from_config();
    }

    // ── Controls with font support ─────────────────────

    fn create_label_wh(&self, text: &str, x: i32, y: i32, w: i32, h: i32, font: HFONT) -> HWND {
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                wstr("STATIC").as_ptr(),
                wstr(text).as_ptr(),
                WS_CHILD | WS_VISIBLE | SS_LEFT_STYLE,
                x,
                y,
                w,
                h,
                self.hwnd,
                0,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        };
        if font != 0 {
            unsafe {
                SendMessageW(hwnd, WM_SETFONT, font as usize, 0);
            }
        }
        hwnd
    }

    fn create_button(&self, text: &str, id: u16, x: i32, y: i32, w: i32, h: i32) -> HWND {
        unsafe {
            CreateWindowExW(
                0,
                wstr("BUTTON").as_ptr(),
                wstr(text).as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
                x,
                y,
                w,
                h,
                self.hwnd,
                id as isize as _,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        }
    }

    fn create_color_button(&self, text: &str, id: u16, x: i32, y: i32, w: i32, h: i32) -> HWND {
        unsafe {
            CreateWindowExW(
                0,
                wstr("BUTTON").as_ptr(),
                wstr(text).as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32 | BS_OWNERDRAW as u32,
                x,
                y,
                w,
                h,
                self.hwnd,
                id as isize as _,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        }
    }

    fn create_checkbox(&self, text: &str, id: u16, x: i32, y: i32, w: i32, h: i32) -> HWND {
        unsafe {
            CreateWindowExW(
                0,
                wstr("BUTTON").as_ptr(),
                wstr(text).as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
                x,
                y,
                w,
                h,
                self.hwnd,
                id as isize as _,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        }
    }

    fn create_combo(&self, id: u16, x: i32, y: i32, w: i32, h: i32) -> HWND {
        unsafe {
            CreateWindowExW(
                0,
                wstr("COMBOBOX").as_ptr(),
                std::ptr::null(),
                WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST as u32,
                x,
                y,
                w,
                h,
                self.hwnd,
                id as isize as _,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        }
    }

    fn create_group(&self, text: &str, x: i32, y: i32, w: i32, h: i32, font: HFONT) -> HWND {
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                wstr("BUTTON").as_ptr(),
                wstr(text).as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_GROUPBOX as u32,
                x,
                y,
                w,
                h,
                self.hwnd,
                0,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        };
        if font != 0 {
            unsafe {
                SendMessageW(hwnd, WM_SETFONT, font as usize, 0);
            }
        }
        hwnd
    }

    // ── Combo ───────────────────────────────────────────

    fn combo_add_string(&self, hwnd: HWND, text: &str) {
        unsafe {
            SendMessageW(hwnd, CB_ADDSTRING, 0, wstr(text).as_ptr() as isize);
        }
    }
    fn combo_select(&self, hwnd: HWND, idx: usize) {
        unsafe {
            SendMessageW(hwnd, CB_SETCURSEL, idx, 0);
        }
    }

    fn populate_effect_combo(&self, t: &crate::i18n::Translations) {
        let h = self.hwnd_effect;
        for s in [
            t.mode_off,
            t.mode_static,
            t.mode_breathing,
            t.mode_single_flash,
            t.mode_double_flash,
            t.mode_multicolor,
            t.mode_rainbow,
        ] {
            self.combo_add_string(h, s);
        }
        self.combo_select(h, self.config.mode.min(6) as usize);
    }
    fn populate_speed_combo(&self, t: &crate::i18n::Translations) {
        let h = self.hwnd_speed;
        for s in [t.speed_slow, t.speed_medium, t.speed_fast] {
            self.combo_add_string(h, s);
        }
        self.combo_select(h, self.config.speed.min(2) as usize);
    }
    fn populate_language_combo(&self, _t: &crate::i18n::Translations) {
        let h = self.hwnd_language;
        for lang in Language::ALL {
            self.combo_add_string(h, lang.display_name());
        }
        let idx = Language::ALL
            .iter()
            .position(|&l| l == self.config.language_enum())
            .unwrap_or(0);
        self.combo_select(h, idx);
    }

    fn refresh_all_texts(&mut self) {
        let t = i18n::translations(self.config.language_enum());
        self.set_text(self.hwnd_header_title, t.brand_title);
        self.set_text(self.hwnd_header_sub, t.brand_subtitle);
        unsafe {
            SendMessageW(self.hwnd_effect, CB_RESETCONTENT, 0, 0);
            SendMessageW(self.hwnd_speed, CB_RESETCONTENT, 0, 0);
            SendMessageW(self.hwnd_language, CB_RESETCONTENT, 0, 0);
        }
        self.populate_effect_combo(&t);
        self.populate_speed_combo(&t);
        self.populate_language_combo(&t);
        self.set_text(self.hwnd_autostart, t.opt_start_boot);
        self.set_text(self.hwnd_close_tray, t.opt_close_tray);
        self.set_text(self.hwnd_start_tray, t.opt_start_in_tray);
        self.set_text(self.get_dlg_item(IDC_SAVE), t.save_apply);
        self.set_text(self.get_dlg_item(IDC_WAKEUP), t.act_force_wakeup);
        self.set_text(self.get_dlg_item(IDC_REFRESH), t.act_refresh);
        self.set_text(self.get_dlg_item(IDC_EXIT), t.act_exit);
        self.update_brightness_label(&t);
    }

    fn set_text(&self, hwnd: HWND, text: &str) {
        unsafe {
            SetWindowTextW(hwnd, wstr(text).as_ptr());
        }
    }
    fn get_dlg_item(&self, id: u16) -> HWND {
        unsafe { GetDlgItem(self.hwnd, id as i32) }
    }

    fn update_controls_from_config(&self) {
        self.combo_select(self.hwnd_effect, self.config.mode.min(6) as usize);
        self.combo_select(self.hwnd_speed, self.config.speed.min(2) as usize);
        unsafe {
            let bp = (3i32 - self.config.brightness.min(3) as i32) as isize;
            SendMessageW(self.hwnd_brightness, TBM_SETPOS, 1, bp);
        }
        self.update_checkboxes();
    }

    fn update_checkboxes(&self) {
        unsafe {
            SendMessageW(self.hwnd_autostart, BM_SETCHECK, self.autostart as usize, 0);
            SendMessageW(
                self.hwnd_close_tray,
                BM_SETCHECK,
                self.config.close_to_tray as usize,
                0,
            );
            SendMessageW(
                self.hwnd_start_tray,
                BM_SETCHECK,
                self.config.start_in_tray as usize,
                0,
            );
        }
    }

    fn update_status_text(&self) {
        self.set_text(self.hwnd_status, &format!("Status: {}", self.status));
    }

    fn update_brightness_label(&self, t: &crate::i18n::Translations) {
        let text = match self.config.brightness {
            0 | 1 => t.speed_slow,
            2 => t.speed_medium,
            _ => t.speed_fast,
        };
        self.set_text(self.hwnd_brightness_label, text);
    }

    fn update_color_button(&mut self, idx: usize) {
        let [r, g, b] = self.config.colors[idx];
        if self.color_brushes[idx] != 0 {
            unsafe {
                DeleteObject(self.color_brushes[idx] as _);
            }
        }
        self.color_brushes[idx] =
            unsafe { CreateSolidBrush((r as u32) | ((g as u32) << 8) | ((b as u32) << 16)) };
        unsafe {
            InvalidateRect(self.hwnd_color_btns[idx], std::ptr::null(), 1);
        }
    }

    fn on_drawitem(&self, id: u16, hdc: HDC, rc: &RECT, item_state: u32) {
        // Find which color button
        if !(IDC_COLOR_BASE..IDC_COLOR_BASE + 7).contains(&id) {
            return;
        }
        let idx = (id - IDC_COLOR_BASE) as usize;

        let [r, g, b] = self.config.colors[idx];
        let bg = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16);
        let sum = r as u32 + g as u32 + b as u32;
        let text_color = if sum > 380 { 0x000000 } else { 0xFFFFFF };

        let pressed = (item_state & 1) != 0; // ODS_SELECTED
        let focused = (item_state & 0x10) != 0; // ODS_FOCUS

        unsafe {
            // Background
            let brush = CreateSolidBrush(bg);
            FillRect(hdc, rc, brush);
            DeleteObject(brush as _);

            // Border: darker shade for 3D effect
            if pressed {
                // Pressed: dark top/left, light bottom/right
                let dark = CreatePen(PS_SOLID, 1, darken(bg, 60));
                let light = CreatePen(PS_SOLID, 1, lighten(bg, 40));
                let old = SelectObject(hdc, dark as _);
                // top
                MoveToEx(hdc, rc.left, rc.bottom - 1, std::ptr::null_mut());
                LineTo(hdc, rc.left, rc.top);
                LineTo(hdc, rc.right - 1, rc.top);
                SelectObject(hdc, light as _);
                LineTo(hdc, rc.right - 1, rc.bottom - 1);
                LineTo(hdc, rc.left, rc.bottom - 1);
                SelectObject(hdc, old);
                DeleteObject(dark as _);
                DeleteObject(light as _);
            } else {
                // Normal: light top/left, dark bottom/right
                let light = CreatePen(PS_SOLID, 1, lighten(bg, 50));
                let dark = CreatePen(PS_SOLID, 1, darken(bg, 50));
                let old = SelectObject(hdc, light as _);
                MoveToEx(hdc, rc.left, rc.bottom - 1, std::ptr::null_mut());
                LineTo(hdc, rc.left, rc.top);
                LineTo(hdc, rc.right - 1, rc.top);
                SelectObject(hdc, dark as _);
                LineTo(hdc, rc.right - 1, rc.bottom - 1);
                LineTo(hdc, rc.left, rc.bottom - 1);
                SelectObject(hdc, old);
                DeleteObject(light as _);
                DeleteObject(dark as _);
            }

            // Focus rectangle
            if focused {
                let mut focus_rc = *rc;
                focus_rc.left += 2;
                focus_rc.top += 2;
                focus_rc.right -= 2;
                focus_rc.bottom -= 2;
                DrawFocusRect(hdc, &focus_rc);
            }

            // Text
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, text_color);
            let label = format!("#{}", idx + 1);
            let wlabel = wstr(&label);
            let mut text_rc = *rc;
            if pressed {
                text_rc.top += 1;
                text_rc.left += 1;
            }
            DrawTextW(
                hdc,
                wlabel.as_ptr(),
                -1,
                &mut text_rc,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
        }
    }

    // ── Events ──────────────────────────────────────────

    fn on_command(&mut self, id: u16, code: u16) {
        match id {
            IDC_EFFECT if code as u32 == CBN_SELCHANGE => {
                let idx = unsafe { SendMessageW(self.hwnd_effect, CB_GETCURSEL, 0, 0) };
                self.config.mode = idx as u8;
            }
            IDC_SPEED if code as u32 == CBN_SELCHANGE => {
                let idx = unsafe { SendMessageW(self.hwnd_speed, CB_GETCURSEL, 0, 0) };
                self.config.speed = idx as u8;
            }
            IDC_LANGUAGE if code as u32 == CBN_SELCHANGE => {
                let idx = unsafe { SendMessageW(self.hwnd_language, CB_GETCURSEL, 0, 0) };
                if let Some(&lang) = Language::ALL
                    .get(idx as usize)
                    .filter(|&&l| self.config.language_enum() != l)
                {
                    self.config.set_language(lang);
                    let _ = save_config(&self.config);
                    self.refresh_all_texts();
                }
            }
            IDC_AUTOSTART => {
                let checked = unsafe { SendMessageW(self.hwnd_autostart, BM_GETCHECK, 0, 0) };
                self.autostart = checked == BST_CHECKED as isize;
                let _ = set_startup(self.autostart);
            }
            IDC_CLOSE_TRAY => {
                let checked = unsafe { SendMessageW(self.hwnd_close_tray, BM_GETCHECK, 0, 0) };
                self.config.close_to_tray = checked == BST_CHECKED as isize;
                let _ = save_config(&self.config);
            }
            IDC_START_TRAY => {
                let checked = unsafe { SendMessageW(self.hwnd_start_tray, BM_GETCHECK, 0, 0) };
                self.config.start_in_tray = checked == BST_CHECKED as isize;
                let _ = save_config(&self.config);
            }
            IDC_SAVE => {
                let t = i18n::translations(self.config.language_enum());
                match save_and_apply(&self.config) {
                    Ok(_) => self.status = t.status_applied.to_string(),
                    Err(e) => self.status = format!("{} {}", t.status_error_prefix, e),
                }
                self.update_status_text();
            }
            IDC_WAKEUP => {
                let t = i18n::translations(self.config.language_enum());
                if recover_keyboard() {
                    self.status = t.status_wakeup_ok.to_string();
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if let Ok(hid) = hidapi::HidApi::new() {
                        let _ = apply_lighting_config(&hid, &self.config);
                    }
                } else {
                    self.status = t.status_wakeup_fail.to_string();
                }
                self.update_status_text();
            }
            IDC_REFRESH => {
                self.status = get_keyboard_status();
                self.autostart = is_startup_enabled();
                self.update_status_text();
                self.update_checkboxes();
            }
            IDC_EXIT => {
                self.cleanup();
                std::process::exit(0);
            }
            _ if (IDC_COLOR_BASE..IDC_COLOR_BASE + 7).contains(&id) => {
                self.pick_color((id - IDC_COLOR_BASE) as usize);
            }
            _ => {}
        }
    }

    fn pick_color(&mut self, idx: usize) {
        let [r, g, b] = self.config.colors[idx];
        let mut custom: [COLORREF; 16] = [0; 16];
        custom[0] = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16);
        unsafe {
            let mut cc: CHOOSECOLORW = std::mem::zeroed();
            cc.lStructSize = std::mem::size_of::<CHOOSECOLORW>() as u32;
            cc.hwndOwner = self.hwnd;
            cc.rgbResult = custom[0];
            cc.lpCustColors = custom.as_mut_ptr();
            cc.Flags = CC_RGBINIT | CC_FULLOPEN;
            if ChooseColorW(&mut cc) != 0 {
                self.config.colors[idx] = [
                    (cc.rgbResult & 0xFF) as u8,
                    ((cc.rgbResult >> 8) & 0xFF) as u8,
                    ((cc.rgbResult >> 16) & 0xFF) as u8,
                ];
                self.update_color_button(idx);
            }
        }
    }

    fn on_hscroll(&mut self, code: u16, _pos: u16, _hwnd: HWND) {
        if code == TB_THUMBPOSITION as u16
            || code == TB_ENDTRACK as u16
            || code == TB_THUMBTRACK as u16
        {
            let pos = unsafe { SendMessageW(self.hwnd_brightness, TBM_GETPOS, 0, 0) };
            self.config.brightness = (3u8).saturating_sub(pos as u8);
            let t = i18n::translations(self.config.language_enum());
            self.update_brightness_label(&t);
        }
    }

    fn on_close(&mut self) {
        let t = i18n::translations(self.config.language_enum());
        if self.config.close_to_tray {
            self.is_visible = false;
            self.status = t.status_tray.to_string();
            self.update_status_text();
            unsafe {
                let mut ex = GetWindowLongW(self.hwnd, GWL_EXSTYLE) as u32;
                ex &= !WS_EX_APPWINDOW;
                ex |= WS_EX_TOOLWINDOW;
                SetWindowLongW(self.hwnd, GWL_EXSTYLE, ex as i32);
                ShowWindow(self.hwnd, SW_HIDE);
            }
        } else {
            self.cleanup();
            unsafe {
                DestroyWindow(self.hwnd);
            }
        }
    }

    fn restore_from_tray(&mut self) {
        if !self.is_visible {
            self.is_visible = true;
            let t = i18n::translations(self.config.language_enum());
            self.status = get_keyboard_status();
            self.autostart = is_startup_enabled();
            self.update_status_text();
            self.update_checkboxes();
            self.update_controls_from_config();
            self.update_brightness_label(&t);
            unsafe {
                let mut ex = GetWindowLongW(self.hwnd, GWL_EXSTYLE) as u32;
                ex &= !WS_EX_TOOLWINDOW;
                ex |= WS_EX_APPWINDOW;
                SetWindowLongW(self.hwnd, GWL_EXSTYLE, ex as i32);
                ShowWindow(self.hwnd, SW_SHOW);
                SetForegroundWindow(self.hwnd);
            }
        }
    }

    fn on_hotkey_toggle(&mut self) {
        self.config = load_config().unwrap_or_default();
        self.update_controls_from_config();
        let t = i18n::translations(self.config.language_enum());
        self.status = t.status_hotkey.to_string();
        self.update_status_text();
    }

    fn cleanup(&mut self) {
        for i in 0..7 {
            if self.color_brushes[i] != 0 {
                unsafe {
                    DeleteObject(self.color_brushes[i] as _);
                }
                self.color_brushes[i] = 0;
            }
        }
        if self.hfont_title != 0 {
            unsafe {
                DeleteObject(self.hfont_title as _);
            }
        }
        if self.hfont_section != 0 {
            unsafe {
                DeleteObject(self.hfont_section as _);
            }
        }
        if self.hfont_normal != 0 {
            unsafe {
                DeleteObject(self.hfont_normal as _);
            }
        }
        if self.hfont_small != 0 {
            unsafe {
                DeleteObject(self.hfont_small as _);
            }
        }
        if self.hbrush_header != 0 {
            unsafe {
                DeleteObject(self.hbrush_header as _);
            }
        }
        if self.hbrush_white != 0 {
            unsafe {
                DeleteObject(self.hbrush_white as _);
            }
        }
    }
}

impl Drop for AppWindow {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// ── Window procedure ────────────────────────────────────────
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        if msg == WM_CREATE {
            let config = load_config().unwrap_or_default();
            let app = Box::new(AppWindow {
                config,
                status: String::new(),
                autostart: false,
                is_visible: false,
                hwnd,
                hfont_title: 0,
                hfont_section: 0,
                hfont_normal: 0,
                hfont_small: 0,
                hbrush_header: 0,
                hbrush_white: 0,
                hwnd_effect: 0,
                hwnd_speed: 0,
                hwnd_brightness: 0,
                hwnd_brightness_label: 0,
                hwnd_color_btns: [0; 7],
                hwnd_language: 0,
                hwnd_autostart: 0,
                hwnd_close_tray: 0,
                hwnd_start_tray: 0,
                hwnd_status: 0,
                hwnd_header_panel: 0,
                hwnd_header_title: 0,
                hwnd_header_sub: 0,
                hwnd_brand_line: 0,
                color_brushes: [0; 7],
            });
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(app) as isize);
            let app = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindow);
            app.init();
            return 0;
        }

        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindow;
        if ptr.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        let app = &mut *ptr;

        match msg {
            WM_COMMAND => {
                app.on_command((wparam & 0xFFFF) as u16, ((wparam >> 16) & 0xFFFF) as u16);
                0
            }
            WM_HSCROLL => {
                app.on_hscroll(
                    (wparam & 0xFFFF) as u16,
                    ((wparam >> 16) & 0xFFFF) as u16,
                    lparam as HWND,
                );
                0
            }
            WM_CTLCOLORSTATIC => {
                let hdc = wparam as HDC;
                let ctrl = lparam as HWND;
                // Header panel: navy background, white text
                if ctrl == app.hwnd_header_panel
                    || ctrl == app.hwnd_header_title
                    || ctrl == app.hwnd_header_sub
                    || ctrl == app.hwnd_brand_line
                {
                    SetBkColor(hdc, COLOR_HEADER_BG);
                    SetTextColor(hdc, 0xFFFFFF);
                    return app.hbrush_header as isize;
                }
                // Status: subtle background based on content
                if ctrl == app.hwnd_status {
                    SetBkColor(hdc, 0xF5F8FA);
                    SetTextColor(hdc, 0x333333);
                    // Return a valid brush
                    return app.hbrush_white as isize;
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_CTLCOLORBTN => {
                // Owner-draw buttons don't use this; kept for non-owner-draw
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            0x002B => {
                // WM_DRAWITEM
                let dis = &*(lparam as *const DRAWITEMSTRUCT);
                if dis.CtlType == 4 {
                    // ODT_BUTTON
                    app.on_drawitem(dis.CtlID as u16, dis.hDC, &dis.rcItem, dis.itemState);
                }
                1 // TRUE
            }
            WM_CLOSE => {
                app.on_close();
                0
            }
            WM_DESTROY => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindow;
                if !ptr.is_null() {
                    let _ = Box::from_raw(ptr);
                }
                PostQuitMessage(0);
                0
            }
            _ if msg == WM_RESTORE_WINDOW => {
                app.restore_from_tray();
                0
            }
            _ if msg == WM_HOTKEY_TOGGLE => {
                app.on_hotkey_toggle();
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

// ── Color helpers ────────────────────────────────────────────

fn darken(color: u32, amount: u8) -> u32 {
    let r = ((color & 0xFF) as i32 - amount as i32).max(0) as u32;
    let g = (((color >> 8) & 0xFF) as i32 - amount as i32).max(0) as u32;
    let b = (((color >> 16) & 0xFF) as i32 - amount as i32).max(0) as u32;
    r | (g << 8) | (b << 16)
}

fn lighten(color: u32, amount: u8) -> u32 {
    let r = ((color & 0xFF) as i32 + amount as i32).min(255) as u32;
    let g = (((color >> 8) & 0xFF) as i32 + amount as i32).min(255) as u32;
    let b = (((color >> 16) & 0xFF) as i32 + amount as i32).min(255) as u32;
    r | (g << 8) | (b << 16)
}
