use eframe::egui;
use egui::ahash::HashMap;
use egui_extras::{Column, TableBuilder};

use super::MCUinterface;
use crate::debugging_tools::*;

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct ToggleSwitchSetting {
    pub off_value: f64,
    pub on_value: f64,
    pub state: bool,
}

impl Default for ToggleSwitchSetting {
    fn default() -> Self {
        Self {
            off_value: 0.0,
            on_value: 1.0,
            state: false,
        }
    }
}

impl ToggleSwitchSetting {
    pub fn value(self) -> f64 {
        if self.state {
            self.on_value
        } else {
            self.off_value
        }
    }

    pub fn set_value(&mut self, value: f64) {
        if value != self.off_value {
            self.state = true;

            self.on_value = self.on_value.max(value);
        }
    }
}
// ----------------------------------------------------------------------------
#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ToggleSwitch {
    mcu: MCUinterface,

    toggle_sw: HashMap<String, ToggleSwitchSetting>,
}

// ----------------------------------------------------------------------------
impl super::WidgetApp for ToggleSwitch {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        const NAME_CLM: f32 = 120.;
        const OFF_CLM: f32 = 50.;
        const TGL_CLM: f32 = 50.;
        const ON_CLM: f32 = 50.;

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .min_scrolled_height(0.0)
                    .column(Column::initial(NAME_CLM).resizable(true))
                    .column(Column::initial(OFF_CLM).resizable(true))
                    .column(Column::initial(TGL_CLM).resizable(true))
                    .column(Column::initial(ON_CLM).resizable(true))
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Symbol Name");
                        });
                        header.col(|ui| {
                            ui.strong("OFF");
                        });
                        header.col(|ui| {
                            ui.strong("Switch");
                        });
                        header.col(|ui| {
                            ui.strong("ON");
                        });
                        header.col(|ui| {
                            ui.strong("Now");
                        });
                    })
                    .body(|mut body| {
                        let view_list = self.mcu.watch_list.clone();
                        for symbol in view_list {
                            let mut tgl = ToggleSwitchSetting::default();
                            if let Some(setting) = self.toggle_sw.get(&symbol.name) {
                                tgl = setting.clone();
                            }
                            let pre_value = tgl.state;

                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(&symbol.name);
                                });
                                row.col(|ui| {
                                    let res =
                                        ui.add(egui::DragValue::new(&mut tgl.off_value).speed(1.0));
                                    if !tgl.state {
                                        res.highlight();
                                    }
                                });
                                row.col(|ui| {
                                    ui.add(toggle(&mut tgl.state));
                                });
                                row.col(|ui| {
                                    let res =
                                        ui.add(egui::DragValue::new(&mut tgl.on_value).speed(1.0));
                                    if tgl.state {
                                        res.highlight();
                                    }
                                });
                                row.col(|ui| {
                                    if let Some(probe) = &mut self.mcu.probe {
                                        if let Some(val) = probe.get_newest_date(&symbol.name) {
                                            ui.label(format!("{}", val));
                                        }
                                    }
                                });

                                if pre_value != tgl.state {
                                    if let Some(probe) = &mut self.mcu.probe {
                                        probe.insert_wirte_que(
                                            &symbol,
                                            format!("{}", &tgl.clone().value()).as_str(),
                                        );
                                    }
                                    //println!("cahge value {}", tgl.value);
                                }
                                self.toggle_sw.insert(symbol.name, tgl.clone());
                            });
                        }
                    });
            });
        });
    }

    fn fetch_watch_list(&mut self, watch_list: &Vec<crate::debugging_tools::VariableInfo>) {
        self.mcu.fetch_watch_list(watch_list);
        for symbol in watch_list {
            self.toggle_sw
                .insert(symbol.name.clone(), ToggleSwitchSetting::default());
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
            let mut tgl = ToggleSwitchSetting::default();
            if let Some(setting) = self.toggle_sw.get(&symbol.name) {
                tgl = setting.clone();
            }

            if let Some(probe) = &mut self.mcu.probe {
                if let Some(val) = probe.get_newest_date(&symbol.name) {
                    tgl.set_value(val);
                }
            }

            self.toggle_sw.insert(symbol.name, tgl.clone());
        }
    }
}

// ----------------------------------------------------------------------------
// from https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/toggle_switch.rs
fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}

pub fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| toggle_ui(ui, on)
}
// ----------------------------------------------------------------------------
