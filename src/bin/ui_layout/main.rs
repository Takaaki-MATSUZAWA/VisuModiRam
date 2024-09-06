#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[path = "../../debugging_tools/mod.rs"]
mod debugging_tools;
#[path = "../../monitor_ui/mod.rs"]
mod monitor_ui;

mod layout_test_monitor;
use layout_test_monitor::LayoutTest;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "VisuModiRam",
        options,
        Box::new(|cc| Box::new(LayoutTest::new(cc))),
    )
}
