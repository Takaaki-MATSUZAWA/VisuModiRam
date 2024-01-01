use eframe::egui;
//use egui_extras::{Column, TableBuilder};

use super::MCUinterface;
use crate::debugging_tools::*;

pub struct WidgetTest {
    name: String,
    age: u32,
    last_data: f64,

    mcu: MCUinterface,
}

impl WidgetTest {
    pub fn new(name: String, age: u32) -> Self {
        Self {
            name,
            age,
            last_data: 0.0,

            mcu: Default::default(),
        }
    }
}

// ----------------------------------------------------------------------------
impl super::WidgetApp for WidgetTest {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
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

            let res = self.mcu.watch_list.first();
            let mut vname: String = "".to_string();
            if let Some(valinfo) = res {
                vname = valinfo.name.clone();
            }
            if vname != "".to_string() {
                let data = if let Some(probe) = &mut self.mcu.probe {
                    probe.get_newest_date(vname.clone())
                } else {
                    None
                };
                if let Some(val) = data {
                    self.last_data = val;
                }

                ui.label(format!("{} data --> {:?}", vname.clone(), self.last_data));
            }
        });
    }

    fn fetch_watch_list(&mut self, watch_list: &Vec<crate::debugging_tools::VariableInfo>) {
        self.mcu.fetch_watch_list(watch_list);
    }

    fn set_probe(&mut self, probe: ProbeInterface2) {
        self.mcu.set_probe(probe);
    }
}
