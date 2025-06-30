// hide console window on Windows in release
#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod debugging_tools;
mod monitor_ui;
mod visumodiram;

use visumodiram::VisuModiRam;

fn main() -> Result<(), eframe::Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting VisuModiRam application");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_drag_and_drop(true)
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon_256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    let version = env!("CARGO_PKG_VERSION");
    let app_title = format!("VisuModiRam v{}", version);

    tracing::info!("version: {}", version);
    
    eframe::run_native(
        &app_title,
        options,
        Box::new(|cc| Ok(Box::new(VisuModiRam::new(cc)))),
    )
}
