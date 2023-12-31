use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::sync::Arc;

use crate::debugging_tools::*;

use super::WidgetApp;

pub struct widgetTest {
    pub name: String,
    pub age: u32,
    watch_list: Option<Arc<Vec<VariableInfo>>>,
    pub probe: Option<Box<ProbeInterface2>>, // Boxを使用して所有権を保持

    last_data: f64,
}

impl widgetTest {
    pub fn new(name: String, age: u32) -> Self {
        Self {
            name,
            age,
            watch_list: None,
            probe: None,
            last_data: 0.0,
        }
    }
}

#[cfg(disapbe)]
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


        let res = self.watch_list.as_ref().and_then(|list| list.first());        
        let mut vname:String = "".to_string();
        if let Some(valinfo) = res {
            vname = valinfo.name.clone();
        }
        if vname != "".to_string(){
            let data = if let Some(probe) = &mut self.probe {
                probe.get_newest_date(vname.clone())
            } else {
                None
            };
            if let Some(val) = data{
                self.last_data = val;
            }

            ui.label(format!("{} data --> {:?}", vname.clone(), self.last_data));
        }
    }

    fn fetch_watch_list(&mut self, watch_list: &Vec<VariableInfo>) {
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

    fn set_probe(&mut self, probe: ProbeInterface2) {
        self.probe = Some(Box::new(probe.clone()));
    }
}
