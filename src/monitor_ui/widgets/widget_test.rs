use eframe::egui;
//use egui_extras::{Column, TableBuilder};

use super::MCUinterface;
use crate::debugging_tools::*;

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WidgetTest {
    name: String,
    age: f64,
    last_data: f64,

    mcu: MCUinterface,
}

// ----------------------------------------------------------------------------
impl super::WidgetApp for WidgetTest {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("STM32EguiMonitor");

            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            if ui.add(egui::Slider::new(&mut self.age, -100.0..=100.0).text("age")).changed(){
                if let Some(probe) = &mut self.mcu.probe {
                    let res = self.mcu.watch_list.first();
                    if let Some(valinfo) = res {
                        probe.insert_wirte_que(valinfo, &self.age.to_string());
                    }
                };           
            };

            if ui.button("Click each year").clicked() {
                self.age += 1.0;
            }

            ui.label(format!("Hello '{}', age {}", self.name, self.age));

            let res = self.mcu.watch_list.first();
            let mut vname: String = "".to_string();
            if let Some(valinfo) = res {
                vname = valinfo.name.clone();
            }
            if vname != "".to_string() {
                let data = if let Some(probe) = &mut self.mcu.probe {
                    probe.get_newest_date(&vname)
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

    fn set_probe(&mut self, probe: ProbeInterface) {
        self.mcu.set_probe(probe);
    }
}
