use eframe::egui::{self, Color32};
use egui_plot::{
    Arrows, AxisHints, Bar, BarChart, BoxElem, BoxPlot, BoxSpread, CoordinatesFormatter, Corner,
    GridInput, GridMark, HLine, Legend, Line, LineStyle, MarkerShape, Plot, PlotImage, PlotPoint,
    PlotPoints, PlotResponse, Points, Polygon, Text, VLine,
};
use std::sync::Arc;

use super::MCUinterface;
use crate::debugging_tools::*;

pub struct GraphMonitor {
    mcu: MCUinterface,
}

impl GraphMonitor {
    pub fn new() -> Self {
        Self {
            mcu: Default::default(),
        }
    }
}

// ----------------------------------------------------------------------------
impl super::WidgetApp for GraphMonitor {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let plot = Plot::new("plot_demo")
            .legend(Legend::default())
            .view_aspect(1.0)
            .y_axis_width(4);

            if let Some(probe) = &mut self.mcu.probe{
                plot.show(ui, |plot_ui| {
                    for val in &mut self.mcu.watch_list.clone(){
                        plot_ui.line({
                            let data: Vec<[f64; 2]> = probe
                                .get_log_vec(val.name.clone())
                                .iter()
                                .enumerate()
                                .map(|(i, y)| [y[0], y[1]])
                                .collect();
                        
                            Line::new(data)
                            .color(Color32::from_rgb(200, 100, 100))
                            .style(LineStyle::Solid)
                            .name(val.name.clone())
                        });
                    }
                });
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
