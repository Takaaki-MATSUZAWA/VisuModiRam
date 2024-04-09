#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[path = "../../debugging_tools/mod.rs"]
mod debugging_tools;
#[path = "../../monitor_ui/mod.rs"]
mod monitor_ui;

mod led_monitor;
use led_monitor::LedMonitor;

use std::{env, path::PathBuf};


fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    // コマンドライン引数からronファイルのパスを取得
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eframe::run_native(
            "LED Controller",
            options,
            Box::new(|cc| Box::new(LedMonitor::new(cc))),
        )
    }else{
        let ron_file_path = &args[1];
        let ron_file_path = PathBuf::from(ron_file_path);
    
        eframe::run_native(
            "LED Controller",
            options,
            Box::new(move |cc| Box::new(LedMonitor::new_with_ronfile(cc, ron_file_path.clone()))),
        )
    }
}
