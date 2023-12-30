#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod debugging_tools;
mod monitor_ui;
mod Egui_monitor;

use Egui_monitor::STM32EguiMonitor;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        //initial_window_size: Some(egui::vec2(960.0, 480.0)),
        initial_window_size: Some([1280.0, 1024.0].into()),
        ..Default::default()
    };
    eframe::run_native(
        "STM32EguiMonitor",
        options,
        Box::new(|_cc| Box::<STM32EguiMonitor>::default()),
    )
}