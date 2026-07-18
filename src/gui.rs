use eframe::egui;
use std::sync::atomic::Ordering;

use crate::config::{KeyboardConfig, load_config, save_config};
use crate::globals::{EGUI_CTX, MAIN_HWND, TOGGLE_FLAG};
use crate::hid::{apply_lighting_config, get_keyboard_status, recover_keyboard, save_and_apply};
use crate::i18n::{self, Language};
use crate::startup::{is_startup_enabled, set_startup};

pub struct KeyboardApp {
    config: KeyboardConfig,
    status: String,
    autostart: bool,
    first_frame: bool,
    is_visible: bool,
}

impl KeyboardApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        *EGUI_CTX.lock().unwrap() = Some(_cc.egui_ctx.clone());

        let did_wakeup = recover_keyboard();
        let config = load_config().unwrap_or_default();

        if did_wakeup {
            let max_retries = 10;
            for attempt in 0..max_retries {
                std::thread::sleep(std::time::Duration::from_millis(if attempt == 0 {
                    300
                } else {
                    500
                }));
                if let Ok(hid) = hidapi::HidApi::new()
                    && apply_lighting_config(&hid, &config).is_ok()
                {
                    break;
                }
            }
        } else if let Ok(hid) = hidapi::HidApi::new() {
            let _ = apply_lighting_config(&hid, &config);
        }

        let status = get_keyboard_status();
        let autostart = is_startup_enabled();
        Self {
            config,
            status,
            autostart,
            first_frame: true,
            is_visible: true,
        }
    }
}

impl eframe::App for KeyboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── 1. Check if we need to restore the window from the tray ──
        if crate::globals::RESTORE_FLAG.swap(false, Ordering::SeqCst) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            self.is_visible = true;
        }

        // ── 2. Check if the window is currently invisible ──
        if !self.is_visible {
            std::thread::sleep(std::time::Duration::from_millis(200));
            return;
        }

        let t = i18n::translations(self.config.language_enum());

        // ── First-frame setup ──────────────────────────
        if self.first_frame {
            self.first_frame = false;
            unsafe {
                let title: Vec<u16> = "Rust Keyboard LED Controller\0".encode_utf16().collect();
                let hwnd = windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW(
                    std::ptr::null(),
                    title.as_ptr(),
                );
                if hwnd != 0 {
                    MAIN_HWND.store(hwnd as u64, Ordering::SeqCst);
                }
            }
        }

        // ── Hotkey / Close-to-tray events ──────────────
        if TOGGLE_FLAG.swap(false, Ordering::SeqCst) {
            self.config = load_config().unwrap_or_default();
            self.status = t.status_hotkey.to_string();
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.config.close_to_tray {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            let h = MAIN_HWND.load(Ordering::SeqCst);
            if h != 0 {
                unsafe {
                    use windows_sys::Win32::UI::WindowsAndMessaging::{
                        GWL_EXSTYLE, GetWindowLongW, SW_MINIMIZE, SetWindowLongW, ShowWindow,
                        WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
                    };
                    let mut ex_style = GetWindowLongW(h as isize, GWL_EXSTYLE) as u32;
                    ex_style &= !WS_EX_APPWINDOW;
                    ex_style |= WS_EX_TOOLWINDOW;
                    SetWindowLongW(h as isize, GWL_EXSTYLE, ex_style as i32);
                    ShowWindow(h as isize, SW_MINIMIZE);
                }
            }
            self.is_visible = false;
            self.status = t.status_tray.to_string();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(t.header);
                });
                ui.separator();

                // Status
                ui.label(format!("Status: {}", self.status));
                ui.add_space(8.0);

                // LED Colors Section
                ui.group(|ui| {
                    ui.label(egui::RichText::new(t.color_config_title).strong());
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        for i in 0..7 {
                            ui.vertical(|ui| {
                                ui.label(format!("#{}", i + 1));
                                ui.color_edit_button_srgb(&mut self.config.colors[i]);
                            });
                        }
                    });
                });
                ui.add_space(8.0);

                // Settings Section
                ui.group(|ui| {
                    ui.label(egui::RichText::new(t.hw_effect_title).strong());
                    ui.add_space(4.0);

                    // Effect Mode
                    ui.horizontal(|ui| {
                        ui.label(t.effect_label);
                        let modes = [
                            (t.mode_off, 0),
                            (t.mode_static, 1),
                            (t.mode_breathing, 2),
                            (t.mode_single_flash, 3),
                            (t.mode_double_flash, 4),
                            (t.mode_multicolor, 5),
                            (t.mode_rainbow, 6),
                        ];
                        let current = modes
                            .iter()
                            .find(|&&(_, v)| v == self.config.mode)
                            .map(|&(n, _)| n)
                            .unwrap_or("Unknown");
                        egui::ComboBox::new("mode_combo", "")
                            .selected_text(current)
                            .show_ui(ui, |ui| {
                                for &(name, val) in &modes {
                                    ui.selectable_value(&mut self.config.mode, val, name);
                                }
                            });
                    });

                    ui.add_space(4.0);
                    if ui.checkbox(&mut self.autostart, t.opt_start_boot).changed() {
                        let _ = set_startup(self.autostart);
                    }
                    if ui
                        .checkbox(&mut self.config.close_to_tray, t.opt_close_tray)
                        .changed()
                    {
                        let _ = save_config(&self.config);
                    }
                });
                ui.add_space(8.0);

                // Language Section
                ui.group(|ui| {
                    ui.label(egui::RichText::new(t.language_label).strong());
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let current_lang = self.config.language_enum();
                        let mut selected_lang = current_lang;
                        egui::ComboBox::new("lang_combo", "")
                            .selected_text(current_lang.display_name())
                            .show_ui(ui, |ui| {
                                for lang in Language::ALL {
                                    ui.selectable_value(
                                        &mut selected_lang,
                                        lang,
                                        lang.display_name(),
                                    );
                                }
                            });
                        if selected_lang != current_lang {
                            self.config.set_language(selected_lang);
                            let _ = save_config(&self.config);
                        }
                    });
                });
                ui.add_space(8.0);

                // Actions Section
                ui.group(|ui| {
                    ui.label(egui::RichText::new(t.actions_title).strong());
                    ui.add_space(4.0);

                    if ui.button(t.save_apply).clicked() {
                        match save_and_apply(&self.config) {
                            Ok(_) => self.status = t.status_applied.to_string(),
                            Err(e) => self.status = format!("{} {}", t.status_error_prefix, e),
                        }
                    }

                    ui.add_space(4.0);
                    if ui.button(t.act_force_wakeup).clicked() {
                        if recover_keyboard() {
                            self.status = t.status_wakeup_ok.to_string();
                        } else {
                            self.status = t.status_wakeup_fail.to_string();
                        }
                    }

                    ui.add_space(4.0);
                    if ui.button(t.act_refresh).clicked() {
                        self.status = get_keyboard_status();
                        self.autostart = is_startup_enabled();
                    }

                    ui.add_space(4.0);
                    if ui.button(t.act_exit).clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });
    }
}
