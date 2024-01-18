use eframe::egui;
use egui::ahash::HashMap;
use egui_extras::{Column, TableBuilder};

use super::MCUinterface;
use crate::debugging_tools::*;

// ----------------------------------------------------------------------------
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct SliderSetting {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub use_steps: bool,
    pub value: f64,
}

impl Default for SliderSetting {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 125.0,
            step: 10.0,
            use_steps: false,
            value: 0.0,
        }
    }
}
// ----------------------------------------------------------------------------
#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Sliders {
    mcu: MCUinterface,

    sliders: HashMap<String, SliderSetting>,
}

// ----------------------------------------------------------------------------
impl super::WidgetApp for Sliders {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        const NAME_CLM: f32 = 120.;
        const SLDR_CLM: f32 = 120.;
        const SENDV_CLM: f32 = 50.;
        const NOWV_CLM: f32 = 50.;
        const RANG_CLM: f32 = 50.;

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .min_scrolled_height(0.0)
                    .column(Column::initial(NAME_CLM).resizable(true))
                    .column(Column::initial(SLDR_CLM).resizable(true))
                    .column(Column::initial(SENDV_CLM).resizable(true))
                    .column(Column::initial(NOWV_CLM).resizable(true))
                    .column(Column::initial(RANG_CLM).resizable(true))
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Symbol Name");
                        });
                        header.col(|ui| {
                            ui.strong("Slider");
                        });
                        header.col(|ui| {
                            ui.strong("Send Value");
                        });
                        header.col(|ui| {
                            ui.strong("Now Value");
                        });
                        header.col(|ui| {
                            ui.strong("Range");
                        });
                        header.col(|ui| {
                            ui.strong("Step");
                        });
                    })
                    .body(|mut body| {
                        let view_list = self.mcu.watch_list.clone();
                        for symbol in view_list {
                            let mut sldr = SliderSetting::default();
                            if let Some(setting) = self.sliders.get(&symbol.name) {
                                sldr = setting.clone();
                            }
                            let pre_value = sldr.value;

                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(&symbol.name);
                                });
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        if ui
                                            .add_enabled(sldr.use_steps, egui::Button::new("◀"))
                                            .clicked()
                                        {
                                            sldr.value -= sldr.step;
                                        };
                                        ui.add(
                                            egui::Slider::new(&mut sldr.value, sldr.min..=sldr.max)
                                                .smart_aim(true)
                                                .show_value(false)
                                                .step_by(sldr.step),
                                        );
                                        if ui
                                            .add_enabled(sldr.use_steps, egui::Button::new("▶"))
                                            .clicked()
                                        {
                                            sldr.value += sldr.step;
                                        };
                                        sldr.value = sldr.value.clamp(sldr.min, sldr.max);
                                    });
                                });
                                row.col(|ui| {
                                    ui.add(egui::DragValue::new(&mut sldr.value).speed(sldr.step));
                                });
                                row.col(|ui| {
                                    if let Some(probe) = &mut self.mcu.probe {
                                        if let Some(val) = probe.get_newest_date(&symbol.name) {
                                            ui.label(format!("{}", val));
                                        }
                                    }
                                });
                                row.col(|ui| {
                                    //  Range
                                    ui.horizontal(|ui| {
                                        let mut min_tmp = sldr.min;
                                        let mut max_tmp = sldr.max;
                                        ui.add(egui::DragValue::new(&mut min_tmp).speed(sldr.step));
                                        ui.label("~");
                                        ui.add(egui::DragValue::new(&mut max_tmp).speed(sldr.step));

                                        if min_tmp < max_tmp {
                                            sldr.min = min_tmp;
                                            sldr.max = max_tmp;
                                        }
                                    });
                                });
                                row.col(|ui| {
                                    //  Step
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut sldr.use_steps, "");
                                        ui.add_enabled(
                                            sldr.use_steps,
                                            egui::DragValue::new(&mut sldr.step).speed(1),
                                        );
                                    });
                                });

                                if pre_value != sldr.value {
                                    if let Some(probe) = &mut self.mcu.probe {
                                        probe.insert_wirte_que(
                                            &symbol,
                                            format!("{}", &sldr.value).as_str(),
                                        );
                                    }
                                    //println!("cahge value {}", sldr.value);
                                }
                                self.sliders.insert(symbol.name, sldr);
                            });
                        }
                    });
            });
        });
    }

    fn fetch_watch_list(&mut self, watch_list: &Vec<crate::debugging_tools::VariableInfo>) {
        self.mcu.fetch_watch_list(watch_list);
        for symbol in watch_list {
            if !self.sliders.contains_key(&symbol.name) {
                self.sliders
                    .insert(symbol.name.clone(), SliderSetting::default());
            }
        }
    }

    fn set_probe(&mut self, probe: ProbeInterface) {
        self.mcu.set_probe(probe);
    }

    fn sync_button_enable(&self) -> bool {
        true
    }

    fn sync(&mut self) {
        let view_list = self.mcu.watch_list.clone();

        for symbol in view_list {
            let mut sldr = SliderSetting::default();
            if let Some(setting) = self.sliders.get(&symbol.name) {
                sldr = setting.clone();
            }

            if let Some(probe) = &mut self.mcu.probe {
                if let Some(val) = probe.get_newest_date(&symbol.name) {
                    sldr.value = val;
                    sldr.max = sldr.max.max(val);
                    sldr.min = sldr.min.min(val);
                }
            }

            self.sliders.insert(symbol.name, sldr);
        }
    }
}
