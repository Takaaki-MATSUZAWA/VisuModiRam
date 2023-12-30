use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::sync::Arc;

use crate::debugging_tools::VariableInfo;

use super::WidgetApp;

pub struct widgetTest {
    pub name: String,
    pub age: u32,
    watch_list: Option<Arc<Vec<VariableInfo>>>,
}

impl widgetTest {
    pub fn new(name: String, age: u32) -> Self {
        Self {
            name,
            age,
            watch_list: None,
        }
    }
}

//#[cfg(disapbe)]
impl eframe::App for widgetTest {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }
}

//#[cfg(disapbe)]
impl<'a> super::WidgetApp<'a> for widgetTest {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("STM32EguiMonitor");

        ui.horizontal(|ui| {
            let name_label = ui.label("Your name: ");
            ui.text_edit_singleline(&mut self.name)
                .labelled_by(name_label.id);
        });
        ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
        if ui.button("Click each year").clicked() {
            self.age += 1;
        }

        ui.label(format!("Hello '{}', age {}", self.name, self.age));
    }

    fn fetch_watch_list(&mut self, watch_list: &Vec<crate::debugging_tools::VariableInfo>) {
        self.watch_list = Some(Arc::new(watch_list.clone()));
    }
}

// ----------------------------------------------------------------------------
#[cfg(enable)]
impl eframe::App for widgetTest {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //self.ui(ui);
            ui.heading("STM32EguiMonitor");

            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }

            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}
impl super::WidgetApp2 for widgetTest {
    fn update(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.ui(ui);
        });
    }

    fn fetch_watch_list(&mut self, watch_list: &Vec<crate::debugging_tools::VariableInfo>) {
        self.watch_list = Some(Arc::new(watch_list.clone()));
    }
}
