use eframe::egui::{self, Color32};
use egui_plot::{Legend, Line, LineStyle, Plot};

use super::MCUinterface;
use crate::debugging_tools::*;

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct GraphMonitor {
    mcu: MCUinterface,
}

// ----------------------------------------------------------------------------
impl super::WidgetApp for GraphMonitor {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let plot = Plot::new("plot_demo")
                .legend(Legend::default())
                .view_aspect(1.0)
                .y_axis_width(4);

            plot.show(ui, |plot_ui| {
                for val in &mut self.mcu.watch_list.clone() {
                    if let Some(probe) = &mut self.mcu.probe {
                        plot_ui.line({
                            let data: Vec<[f64; 2]> = probe
                                .get_log_vec(&val.name)
                                .iter()
                                .enumerate()
                                .map(|(_i, y)| [y[0], y[1]])
                                .collect();

                            Line::new(data)
                                .color(Color32::from_rgb(200, 100, 100))
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
