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
            .with_inner_size([350.0, 520.0])
            .with_min_inner_size([300.0, 400.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "Rust Keyboard LED Controller",
        options,
        Box::new(|cc| Ok(Box::new(gui::KeyboardApp::new(cc)) as Box<dyn eframe::App>)),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
