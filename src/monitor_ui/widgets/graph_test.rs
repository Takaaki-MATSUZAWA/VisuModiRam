use crate::STM32EguiMonitor;
use eframe::egui::{self, Color32};
use egui_extras::{Column, TableBuilder};

use egui_plot::{
    Arrows, AxisHints, Bar, BarChart, BoxElem, BoxPlot, BoxSpread, CoordinatesFormatter, Corner,
    GridInput, GridMark, HLine, Legend, Line, LineStyle, MarkerShape, Plot, PlotImage, PlotPoint,
    PlotPoints, PlotResponse, Points, Polygon, Text, VLine,
};

use crate::debugging_tools::ProbeInterface;


pub struct GraphTest {
    pub name: String,
    pub my_probe: Option<Box<ProbeInterface>>, // Boxを使用して所有権を保持
    time: f64,
}

impl GraphTest{
    pub fn new() -> Self {
        Self {
            name: "Graph Windowe test".to_string(),
            my_probe: None,
            time: 0.0,
        }
    }

    pub fn ui(&mut self, probe_if:&mut ProbeInterface, ui: &mut egui::Ui, ctx: &egui::Context, frame: &eframe::Frame){
        use std::f32::consts::{PI, TAU};
        self.time = ctx.input(|input_state| input_state.time);

        let plot = Plot::new("plot_demo")
            .legend(Legend::default())
            .view_aspect(1.0)
            .y_axis_width(4);

        plot.show(ui, |plot_ui| {
            plot_ui.line({
                let n = 512;
                let circle_points: PlotPoints = (0..=n)
                    .map(|i| {
                        let t = remap(i as f64, 0.0..=(n as f64), 0.0..=TAU.into());
                        //let r = self.circle_radius;
                        let r = 1.0;
                        [
                            //r * t.cos() + self.circle_center.x as f64,
                            //r * t.sin() + self.circle_center.y as f64,
                            r * t.cos() as f64,
                            r * t.sin() as f64,
                        ]
                    })
                    .collect();
                Line::new(circle_points)
                    .color(Color32::from_rgb(100, 200, 100))
                    .style(LineStyle::Solid)
                    .name("circle")
            });
            plot_ui.line({
                let time = self.time;
                Line::new(PlotPoints::from_explicit_callback(
                    move |x| 0.5 * (2.0 * x).sin() * time.sin(),
                    ..,
                    512,
                ))
                .color(Color32::from_rgb(200, 100, 100))
                .style(LineStyle::Solid)
                .name("wave")
            });

            plot_ui.line({
                let data: Vec<[f64; 2]> = probe_if
                    .get_log_vec()
                    .iter()
                    .enumerate()
                    .map(|(i, y)| [y[0], y[1]])
                    .collect();

                Line::new(data)
                    .color(Color32::from_rgb(200, 100, 100))
                    .style(LineStyle::Solid)
                    .name("wave")
            });
        });
    }
}

fn remap(
    value: f64,
    from_range: std::ops::RangeInclusive<f64>,
    to_range: std::ops::RangeInclusive<f64>,
) -> f64 {
    let (from_min, from_max) = (*from_range.start(), *from_range.end());
    let (to_min, to_max) = (*to_range.start(), *to_range.end());

    to_min + (value - from_min) * (to_max - to_min) / (from_max - from_min)
}