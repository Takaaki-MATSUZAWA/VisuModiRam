// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod debugging_tools;
mod egui_monitor;
mod monitor_ui;

use egui_monitor::STM32EguiMonitor;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        //initial_window_size: Some(egui::vec2(960.0, 480.0)),
        initial_window_size: Some([1280.0, 720.0].into()),
        ..Default::default()
    };
    eframe::run_native(
        "STM32EguiMonitor",
        options,
        Box::new(|cc| Box::new(STM32EguiMonitor::new(cc))),
    )
}
