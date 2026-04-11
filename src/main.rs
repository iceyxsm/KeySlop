#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod autostart;
mod config;
mod listener;
mod tray;

use app::KeySlopApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 600.0])
            .with_min_inner_size([400.0, 400.0])
            .with_title("KeySlop")
            .with_close_button(true),
        ..Default::default()
    };

    eframe::run_native(
        "KeySlop",
        native_options,
        Box::new(|cc| Ok(Box::new(KeySlopApp::new(cc)))),
    )
}
