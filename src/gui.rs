use eframe::egui;
use std::sync::atomic::Ordering;

use crate::config::{KeyboardConfig, get_config_path, load_config, save_config};
use crate::globals::{EGUI_CTX, LAST_VK, MAIN_HWND, TOGGLE_FLAG};
use crate::hid::{apply_lighting_config, get_keyboard_status, recover_keyboard, save_and_apply};
use crate::i18n::{self, Language};
use crate::startup::{is_startup_enabled, set_startup};

// ── Design tokens ─────────────────────────────────────────

const ACCENT: egui::Color32 = egui::Color32::from_rgb(139, 92, 246);
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(22, 22, 26);
const CARD_BORDER: egui::Color32 = egui::Color32::from_rgb(37, 37, 43);
const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(113, 113, 122);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(228, 228, 231);
const DANGER: egui::Color32 = egui::Color32::from_rgb(239, 68, 68);

// ── Helpers ────────────────────────────────────────────────

/// Draw a section card with a left-accent header, then body content inside the card
fn section_card<R>(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Frame::none()
        .fill(CARD_BG)
        .rounding(10.0)
        .inner_margin(egui::vec2(18.0, 16.0))
        .outer_margin(egui::vec2(0.0, 6.0))
        .stroke(egui::Stroke::new(1.0, CARD_BORDER))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Section header with accent bar
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(3.0, 18.0), egui::Sense::hover());
                ui.painter_at(rect)
                    .rect_filled(rect, egui::Rounding::same(2.0), ACCENT);
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(title)
                        .size(14.0)
                        .strong()
                        .color(TEXT_PRIMARY),
                );
            });
            ui.add_space(8.0);
            add_contents(ui)
        })
}

// ── GUI Application ───────────────────────────────────────

pub struct KeyboardApp {
    config: KeyboardConfig,
    status: String,
    autostart: bool,
    first_frame: bool,
}

impl KeyboardApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        *EGUI_CTX.lock().unwrap() = Some(_cc.egui_ctx.clone());

        // Keyboard init — window is already visible by now, no black flash
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
        }
    }
}

impl eframe::App for KeyboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Limit frame rate to maximum 30 FPS to prevent high CPU usage on high-refresh-rate screens or when V-Sync is disabled
        static mut LAST_FRAME_TIME: Option<std::time::Instant> = None;
        unsafe {
            let now = std::time::Instant::now();
            if let Some(last) = LAST_FRAME_TIME {
                let elapsed = now.duration_since(last);
                let min_frame_duration = std::time::Duration::from_millis(33); // ~30 FPS limit
                if elapsed < min_frame_duration {
                    std::thread::sleep(min_frame_duration - elapsed);
                }
            }
            LAST_FRAME_TIME = Some(std::time::Instant::now());
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
        if crate::globals::RESTORE_FLAG.swap(false, Ordering::SeqCst) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.config.close_to_tray {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.status = t.status_tray.to_string();
        }

        // ═══════════════════════════════════════════════
        // SINGLE UNIFIED LAYOUT
        // ═══════════════════════════════════════════════
        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(20.0))
            .show(ctx, |ui| {
                // ── Top brand bar ────────────────────
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("\u{2328}  {}", t.brand_title))
                            .size(15.0)
                            .strong()
                            .color(ACCENT),
                    );
                    ui.label(
                        egui::RichText::new(t.brand_subtitle)
                            .size(11.0)
                            .color(TEXT_MUTED),
                    );
                });
                ui.separator();
                ui.add_space(2.0);

                // ── Header ──────────────────────────
                ui.label(egui::RichText::new(t.header).size(20.0).strong());
                ui.add_space(6.0);

                // ── Two-column body ──────────────────
                ui.columns(2, |cols| {
                    // ── LEFT COLUMN: config cards ─────
                    egui::ScrollArea::vertical().show(&mut cols[0], |ui| {
                        // ── HARDWARE EFFECT CARD ────
                        section_card(ui, t.hw_effect_title, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(t.effect_label);
                                ui.add_space(8.0);
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
                                    .width(200.0)
                                    .show_ui(ui, |ui| {
                                        for &(name, val) in &modes {
                                            ui.selectable_value(&mut self.config.mode, val, name);
                                        }
                                    });
                            });

                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                ui.label(t.speed_label);
                                ui.add_space(8.0);
                                ui.add(egui::Slider::new(&mut self.config.speed, 0..=2).text(""));
                                ui.add_space(6.0);
                                ui.label(
                                    egui::RichText::new(match self.config.speed {
                                        0 => t.speed_slow,
                                        1 => t.speed_medium,
                                        _ => t.speed_fast,
                                    })
                                    .size(12.0)
                                    .color(TEXT_MUTED),
                                );
                            });

                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                ui.label(t.brightness_label);
                                ui.add_space(8.0);
                                ui.add(
                                    egui::Slider::new(&mut self.config.brightness, 0..=10).text(""),
                                );
                                ui.add_space(6.0);
                                let level = self.config.brightness;
                                let (bar_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(110.0, 14.0),
                                    egui::Sense::hover(),
                                );
                                let p = ui.painter_at(bar_rect);
                                let seg_w = bar_rect.width() / 10.0;
                                for i in 0..10 {
                                    let x = bar_rect.left() + i as f32 * seg_w;
                                    let seg = egui::Rect::from_min_size(
                                        egui::pos2(x + 1.0, bar_rect.top() + 1.0),
                                        egui::vec2(seg_w - 2.0, bar_rect.height() - 2.0),
                                    );
                                    let alpha: u8 = if i < level { 200 + (i * 5) } else { 30 };
                                    p.rect_filled(
                                        seg,
                                        egui::Rounding::same(2.0),
                                        egui::Color32::from_rgba_premultiplied(139, 92, 246, alpha),
                                    );
                                }
                            });
                        });

                        // ── COLOR CONFIGURATION CARD ─
                        section_card(ui, t.color_config_title, |ui| {
                            ui.horizontal(|ui| {
                                for i in 0..7 {
                                    let c = self.config.colors[i];
                                    let mut edit_rgb = egui::Color32::from_rgb(c[0], c[1], c[2]);
                                    if ui.color_edit_button_srgba(&mut edit_rgb).changed() {
                                        self.config.colors[i] =
                                            [edit_rgb.r(), edit_rgb.g(), edit_rgb.b()];
                                    }
                                }
                            });
                        });

                        // ── SAVE BUTTON ──────────────
                        let save_btn = ui.add_sized(
                            [ui.available_width(), 40.0],
                            egui::Button::new(
                                egui::RichText::new(t.save_apply).size(15.0).strong(),
                            )
                            .fill(ACCENT)
                            .rounding(10.0),
                        );

                        if save_btn.clicked() {
                            match save_and_apply(&self.config) {
                                Ok(_) => self.status = t.status_applied.to_string(),
                                Err(e) => self.status = format!("{} {}", t.status_error_prefix, e),
                            }
                        }
                        ui.add_space(8.0);
                    });

                    // ── RIGHT COLUMN: options & actions ─
                    egui::ScrollArea::vertical().show(&mut cols[1], |ui| {
                        // ── Info card (top) ─────
                        section_card(ui, t.info_title, |ui| {
                            let win_size = ctx.screen_rect().size();
                            ui.label(
                                egui::RichText::new(format!(
                                    "\u{1F5A5}  {} x {}",
                                    win_size.x as i32, win_size.y as i32
                                ))
                                .size(11.0)
                                .color(TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(format!("\u{1F4C1}  {}", get_config_path()))
                                    .size(11.0)
                                    .color(TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            let last_vk = LAST_VK.load(Ordering::SeqCst);
                            let shortcut_text = if last_vk != 0 {
                                format!("{}  (VK: 0x{:X})", t.fn_f8_info, last_vk)
                            } else {
                                t.fn_f8_info.to_string()
                            };
                            ui.label(
                                egui::RichText::new(shortcut_text)
                                    .size(11.0)
                                    .color(TEXT_MUTED),
                            );
                        });

                        // ── Options card ────────
                        section_card(ui, t.options_title, |ui| {
                            // Language selector
                            ui.horizontal(|ui| {
                                ui.label(t.language_label);
                                ui.add_space(4.0);
                                let current_lang = self.config.language_enum();
                                let mut selected_lang = current_lang;
                                egui::ComboBox::new("lang_combo", "")
                                    .selected_text(current_lang.display_name())
                                    .width(120.0)
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
                            ui.add_space(3.0);
                            if ui.checkbox(&mut self.autostart, t.opt_start_boot).changed() {
                                let _ = set_startup(self.autostart);
                            }
                            ui.add_space(3.0);
                            if ui
                                .checkbox(&mut self.config.close_to_tray, t.opt_close_tray)
                                .changed()
                            {
                                let _ = save_config(&self.config);
                            }
                        });

                        // ── Actions card ─────────
                        section_card(ui, t.actions_title, |ui| {
                            if ui
                                .add_sized(
                                    [ui.available_width(), 28.0],
                                    egui::Button::new(
                                        egui::RichText::new(t.act_force_wakeup).size(13.0),
                                    ),
                                )
                                .clicked()
                            {
                                if recover_keyboard() {
                                    self.status = t.status_wakeup_ok.to_string();
                                } else {
                                    self.status = t.status_wakeup_fail.to_string();
                                }
                            }
                            ui.add_space(6.0);
                            if ui
                                .add_sized(
                                    [ui.available_width(), 26.0],
                                    egui::Button::new(
                                        egui::RichText::new(t.act_refresh).size(12.0),
                                    ),
                                )
                                .clicked()
                            {
                                self.status = get_keyboard_status();
                                self.autostart = is_startup_enabled();
                            }
                            ui.add_space(6.0);
                            if ui
                                .add_sized(
                                    [ui.available_width(), 28.0],
                                    egui::Button::new(
                                        egui::RichText::new(t.act_exit).size(12.0).color(DANGER),
                                    )
                                    .fill(egui::Color32::from_rgb(40, 16, 16)),
                                )
                                .clicked()
                            {
                                std::process::exit(0);
                            }
                        });
                    });
                });
            });
    }
}
