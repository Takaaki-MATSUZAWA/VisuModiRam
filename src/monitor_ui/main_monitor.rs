use eframe::egui::{self, Button, Color32};
use egui_extras::{Column, TableBuilder};

#[derive(Default)]
pub struct MainMonitorTab {}

impl eframe::App for MainMonitorTab {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //let window_width = ctx.available_rect().width();

        egui::SidePanel::left("control")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("probe control panel");
                ui.separator();
                ui.label("text 1");
            });

        egui::SidePanel::right("widgets")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("monitor app list");
                ui.separator();
                ui.label("text 1");
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("main panel");
        });
    }
}
