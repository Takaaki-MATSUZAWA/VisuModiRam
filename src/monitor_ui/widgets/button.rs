use eframe::egui;
use egui_extras::{Column, TableBuilder};

use super::MCUinterface;
use crate::debugging_tools::*;

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct ButtonInfo {
    id: u32,
    name: String,
    symbol_name: String,
    send_value: f64,
}

impl ButtonInfo {
    pub fn new(id: u32) -> Self {
        let mut slf = Self::default();
        slf.id = id;
        slf.name = format!("btn_{}", id);
        slf
    }
}
// ----------------------------------------------------------------------------
#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PushButton {
    mcu: MCUinterface,

    buttons: Vec<ButtonInfo>,
    btn_cnt: u32,
}

// ----------------------------------------------------------------------------
impl super::WidgetApp for PushButton {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        const NAME_CLM: f32 = 50.;
        const SYML_CLM: f32 = 120.;
        const SEND_CLM: f32 = 40.;
        const BTN_CLM: f32 = 70.;

        if ui.button("Add Push Button").clicked() {
            self.buttons.push(ButtonInfo::new(self.btn_cnt));
            self.btn_cnt += 1;
        };

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .min_scrolled_height(0.0)
                    .column(Column::initial(NAME_CLM).resizable(true))
                    .column(Column::initial(BTN_CLM).resizable(true))
                    .column(Column::initial(SEND_CLM).resizable(true))
                    .column(Column::initial(SYML_CLM).resizable(true))
                    .column(Column::remainder()) // remove
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Name");
                        });
                        header.col(|ui| {
                            ui.strong("Button");
                        });
                        header.col(|ui| {
                            ui.strong("Value");
                        });
                        header.col(|ui| {
                            ui.strong("Synbol");
                        });
                        header.col(|_| {});
                    })
                    .body(|mut body| {
                        let mut remove_que = None;

                        for btn in &mut self.buttons {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.text_edit_singleline(&mut btn.name);
                                });
                                row.col(|ui| {
                                    if ui.button("Send Value").clicked() {
                                        if let Some(probe) = &mut self.mcu.probe {
                                            if let Some(symbol) = self
                                                .mcu
                                                .watch_list
                                                .iter()
                                                .find(|w| w.name == btn.symbol_name)
                                            {
                                                probe.insert_wirte_que(
                                                    &symbol,
                                                    format!("{}", &btn.send_value).as_str(),
                                                );
                                            }
                                        }
                                    }
                                });
                                row.col(|ui| {
                                    ui.add(egui::DragValue::new(&mut btn.send_value).speed(1.0));
                                });
                                row.col(|ui| {
                                    egui::ComboBox::from_id_source(btn.id)
                                        .selected_text(btn.symbol_name.clone())
                                        .show_ui(ui, |ui| {
                                            let mut index = 0;
                                            let res = self
                                                .mcu
                                                .watch_list
                                                .iter()
                                                .position(|w| w.name == btn.symbol_name);
                                            if let Some(_index) = res {
                                                index = _index;
                                            }
                                            for i in 0..self.mcu.watch_list.len() {
                                                let symbol = &self.mcu.watch_list[i];
                                                ui.selectable_value(
                                                    &mut index,
                                                    i,
                                                    symbol.name.clone(),
                                                );
                                            }
                                            if self.mcu.watch_list.len() > 0 {
                                                btn.symbol_name =
                                                    self.mcu.watch_list[index].name.clone();
                                            }
                                        });
                                });
                                row.col(|ui| {
                                    if ui.button("X").clicked() {
                                        remove_que = Some(btn.id);
                                    }
                                });
                            });
                        }
                        if let Some(id) = remove_que {
                            self.buttons.retain(|x| x.id != id);
                        }
                    });
            });
        });
    }

    fn fetch_watch_list(&mut self, watch_list: &Vec<crate::debugging_tools::VariableInfo>) {
        self.mcu.fetch_watch_list(watch_list);
    }

    fn set_probe(&mut self, probe: ProbeInterface) {
        self.mcu.set_probe(probe);
    }
}
