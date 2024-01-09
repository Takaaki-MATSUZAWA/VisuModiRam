use eframe::egui;
use egui::{ahash::HashMap, Align, Color32, Direction, Layout};

use super::MCUinterface;
use crate::debugging_tools::*;
use egui_gauge::Gauge;

// ----------------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct LayoutSettings {
    // Similar to the contents of `egui::Layout`
    main_dir: Direction,
    main_wrap: bool,
    cross_align: Align,
    cross_justify: bool,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self::top_down()
    }
}

impl LayoutSettings {
    fn top_down() -> Self {
        Self {
            main_dir: Direction::TopDown,
            main_wrap: false,
            cross_align: Align::Min,
            cross_justify: false,
        }
    }

    fn horizontal_wrapped() -> Self {
        Self {
            main_dir: Direction::LeftToRight,
            main_wrap: true,
            cross_align: Align::Center,
            cross_justify: false,
        }
    }

    fn layout(&self) -> Layout {
        Layout::from_main_dir_and_cross_align(self.main_dir, self.cross_align)
            .with_main_wrap(self.main_wrap)
            .with_cross_justify(self.cross_justify)
    }
}
// ----------------------------------------------------------------------------
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct GaugeSetting {
    pub min: f64,
    pub max: f64,
    pub size: f32,
    pub value: f64,
}

impl Default for GaugeSetting {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 125.0,
            size: 200.0,
            value: 0.0,
        }
    }
}
// ----------------------------------------------------------------------------
#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Gauges {
    mcu: MCUinterface,

    sliders: HashMap<String, GaugeSetting>,
    layout: LayoutSettings,
    common_size: f32,
}

// ----------------------------------------------------------------------------
impl super::WidgetApp for Gauges {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.label("Dirction :");
            ui.selectable_value(&mut self.layout, LayoutSettings::top_down(), "Vertical");
            ui.selectable_value(
                &mut self.layout,
                LayoutSettings::horizontal_wrapped(),
                "Horizontal",
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Set all").clicked() {
                    for slider in self.sliders.values_mut() {
                        slider.size = self.common_size;
                    }
                }
                ui.add(egui::DragValue::new(&mut self.common_size).clamp_range(50..=500));
                ui.label("size");
                ui.separator();
            });
        });

        ui.with_layout(self.layout.layout(), |ui| {
            let view_list = self.mcu.watch_list.clone();
            for symbol in view_list {
                let mut sldr = GaugeSetting::default();
                if let Some(setting) = self.sliders.get(&symbol.name) {
                    sldr = setting.clone();
                }

                if let Some(probe) = &mut self.mcu.probe {
                    if let Some(val) = probe.get_newest_date(&symbol.name) {
                        sldr.value = val;
                    }
                }

                ui.vertical(|ui| {
                    ui.collapsing(format!("{}_setting", symbol.name.clone()), |ui| {
                        ui.horizontal(|ui| {
                            let mut min_tmp = sldr.min;
                            let mut max_tmp = sldr.max;
                            let mut size_tmp = sldr.size;
                            ui.label("range:");
                            ui.add(egui::DragValue::new(&mut min_tmp));
                            ui.label("~");
                            ui.add(egui::DragValue::new(&mut max_tmp));
                            ui.separator();
                            ui.label("size:");
                            ui.add(
                                egui::DragValue::new(&mut size_tmp)
                                    .clamp_range(50..=500)
                                    .speed(1),
                            );

                            if min_tmp < max_tmp {
                                sldr.min = min_tmp;
                                sldr.max = max_tmp;
                            }
                            sldr.size = size_tmp;
                        });
                    });
                    let view_val = self.round(sldr.value);
                    ui.add(
                        Gauge::new(view_val, sldr.min..=sldr.max, sldr.size, Color32::RED)
                            .text(symbol.name.clone()),
                    );
                });
                self.sliders.insert(symbol.name, sldr);
            }
        });
    }

    fn fetch_watch_list(&mut self, watch_list: &Vec<crate::debugging_tools::VariableInfo>) {
        self.mcu.fetch_watch_list(watch_list);
        for symbol in watch_list {
            self.sliders
                .insert(symbol.name.clone(), GaugeSetting::default());
        }
    }

    fn set_probe(&mut self, probe: ProbeInterface) {
        self.mcu.set_probe(probe);
    }

    fn disalbe_scroll_area(&self) -> bool {
        true
    }
}

impl Gauges {
    fn round(&mut self, val: f64) -> f64 {
        let tmp = (val * 100.0).round();
        tmp / 100.0
    }
}
