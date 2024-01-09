use eframe::egui;
use egui_plot::{Corner, Legend, Line, LineStyle, Plot};

use super::MCUinterface;
use crate::debugging_tools::*;

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct GraphMonitor {
    mcu: MCUinterface,

    time_window: u64,
    entire_duration_flag: bool,
}

impl Default for GraphMonitor {
    fn default() -> Self {
        Self {
            mcu: Default::default(),
            time_window: 10000,
            entire_duration_flag: false,
        }
    }
}

// ----------------------------------------------------------------------------
impl super::WidgetApp for GraphMonitor {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let mut plot = Plot::new("plot_demo")
                .legend(Legend::default().position(Corner::LeftTop))
                //.view_aspect(1.0)
                .y_axis_width(4);

            let mut reset_flag = false;
            ui.horizontal(|ui| {
                if ui.button("Pos Reset").clicked() {
                    reset_flag = true;
                };
                ui.separator();
                ui.add_enabled(self.entire_duration_flag == false, {
                    egui::Slider::new(&mut self.time_window, 500..=20000)
                        .smart_aim(true)
                        .step_by(200.)
                });
                ui.label("[ms]");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .selectable_label(self.entire_duration_flag, "All time")
                        .clicked()
                    {
                        self.entire_duration_flag = !self.entire_duration_flag;
                    }
                    ui.separator();
                });
            });

            if reset_flag {
                plot = plot.reset();
            }

            plot.show(ui, |plot_ui| {
                for val in &mut self.mcu.watch_list.clone() {
                    if let Some(probe) = &mut self.mcu.probe {
                        let time_window = if self.entire_duration_flag {
                            None
                        } else {
                            Some(self.time_window)
                        };
                        plot_ui.line({
                            let data: Vec<[f64; 2]> = probe
                                .get_log_vec(&val.name, time_window)
                                .iter()
                                .enumerate()
                                .map(|(_i, y)| [y[0], y[1]])
                                .collect();

                            Line::new(data)
                                //.color(Color32::from_rgb(200, 100, 100))
                                .style(LineStyle::Solid)
                                .name(val.name.clone())
                        });
                    }
                }
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
