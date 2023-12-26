use std::fmt::format;

use eframe::egui::{self, Button, Color32};
use egui_extras::{Column, TableBuilder};

use super::ProbeSetting;
use super::SymbolSearch;

#[derive(Default)]
pub struct SettingTab {
    probe_setting_ui: ProbeSetting,
    symbol_serch_ui: SymbolSearch,
}

impl eframe::App for SettingTab {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let window_width = ctx.available_rect().width();

        egui::SidePanel::left("elf parser")
            //.resizable(true)
            .exact_width(window_width / 2.0)
            .show(ctx, |ui| {
                //ui.heading("elf info");
                //ui.separator();
                //ui.label(format!("widnow width :{}", window_width));
                self.symbol_serch_ui.ui(ctx, ui, frame);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.heading("probe setting");
            self.probe_setting_ui.ui(ui, frame);
        });
    }
}
