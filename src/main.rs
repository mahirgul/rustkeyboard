#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod globals;
mod gui;
mod hid;
mod hook;
mod i18n;
mod startup;
mod tray;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    hook::start_keyboard_hook();
    tray::start_tray_icon_thread();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Rust Keyboard LED Controller")
            .with_inner_size([860.0, 620.0])
            .with_min_inner_size([860.0, 620.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "Rust Keyboard LED Controller",
        options,
        Box::new(|cc| {
            let mut visuals = eframe::egui::Visuals::dark();
            visuals.window_rounding = eframe::egui::Rounding::same(8.0);
            visuals.widgets.noninteractive.rounding = eframe::egui::Rounding::same(8.0);
            visuals.widgets.inactive.rounding = eframe::egui::Rounding::same(8.0);
            visuals.widgets.hovered.rounding = eframe::egui::Rounding::same(8.0);
            visuals.widgets.active.rounding = eframe::egui::Rounding::same(8.0);
            visuals.widgets.open.rounding = eframe::egui::Rounding::same(8.0);

            // ── Deep charcoal dark theme ──
            visuals.extreme_bg_color = eframe::egui::Color32::from_rgb(8, 8, 10); // deepest
            visuals.window_fill = eframe::egui::Color32::from_rgb(17, 17, 19); // main bg
            visuals.widgets.inactive.weak_bg_fill = eframe::egui::Color32::from_rgb(24, 24, 28);
            visuals.widgets.hovered.weak_bg_fill = eframe::egui::Color32::from_rgb(32, 32, 37);
            visuals.widgets.active.weak_bg_fill = eframe::egui::Color32::from_rgb(38, 38, 44);

            // Purple accent
            visuals.selection.bg_fill = eframe::egui::Color32::from_rgb(139, 92, 246);

            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::new(gui::KeyboardApp::new(cc)) as Box<dyn eframe::App>)
        }),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
