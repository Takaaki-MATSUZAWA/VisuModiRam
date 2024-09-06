// hide console window on Windows in release
#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod debugging_tools;
mod egui_monitor;
mod monitor_ui;

use egui_monitor::VisuModiRam;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "VisuModiRam",
        options,
        Box::new(|cc| Box::new(VisuModiRam::new(cc))),
    )
}
