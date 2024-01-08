use eframe::egui;
use egui::ahash::HashMap;
use egui_extras::{Column, TableBuilder};

use super::MCUinterface;
use crate::debugging_tools::*;

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct EditTable {
    mcu: MCUinterface,

    edit_texts: HashMap<String, String>,
}

// ----------------------------------------------------------------------------
impl super::WidgetApp for EditTable {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        const NAME_CLM: f32 = 120.;
        const EDIT_CLM: f32 = 100.;

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .min_scrolled_height(0.0)
                    .column(Column::initial(NAME_CLM).resizable(true))
                    .column(Column::initial(EDIT_CLM).resizable(true))
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Symbol Name");
                        });
                        header.col(|ui| {
                            ui.strong("Edit Value");
                        });
                        header.col(|ui| {
                            ui.strong("Now Value");
                        });
                    })
                    .body(|mut body| {
                        let view_list = self.mcu.watch_list.clone();
                        for symbol in view_list {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(&symbol.name);
                                });
                                row.col(|ui| {
                                    let mut text = "".to_string();
                                    let res = self.edit_texts.get(&symbol.name);
                                    if let Some(_text) = res {
                                        text = _text.clone();
                                    }
                                    let res = ui.text_edit_singleline(&mut text);
                                    self.edit_texts.insert(symbol.name.clone(), text.clone());

                                    if res.changed() && text != "" {
                                        if let Some(probe) = &mut self.mcu.probe {
                                            probe.insert_wirte_que(&symbol, &text);
                                        }
                                    }
                                });
                                row.col(|ui| {
                                    if let Some(probe) = &mut self.mcu.probe {
                                        if let Some(val) = probe.get_newest_date(&symbol.name) {
                                            ui.label(format!("{}", val));
                                        }
                                    }
                                });
                            });
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
