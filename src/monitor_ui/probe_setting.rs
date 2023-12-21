use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::debugging_tools::ProbeInterface;

pub struct ProbeSetting{
    probes: Vec<probe_rs::DebugProbeInfo>,
    my_probe: ProbeInterface,
}

impl Default for ProbeSetting{
    fn default() -> Self{
        ProbeSetting::none()
    }
}
impl ProbeSetting{
    fn none() -> Self {
        Self {
            probes: Vec::new(),
            my_probe: Default::default(),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {

    }

    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.heading("ELF File Viewer");

        if ui.add(egui::Button::new("prove check")).clicked() {
            self.probes = Vec::new();
            self.probes = self.my_probe.get_connected_probes();
        }
        ui.push_id(0, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .max_scroll_height(40.)
                .vscroll(true)
                .drag_to_scroll(true)
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("name");
                    });

                    header.col(|ui| {
                        ui.heading("type");
                    });
                    header.col(|ui| {
                        ui.heading("vnd_id");
                    });
                    header.col(|ui| {
                        ui.heading("prd_id");
                    });
                    header.col(|ui| {
                        ui.heading("SN");
                    });
                })
                .body(|mut body| {
                    for probe in &self.probes {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.label(&probe.identifier);
                            });
                            row.col(|ui| {
                                ui.label(format!("{:?}", probe.probe_type));
                            });
                            row.col(|ui| {
                                ui.label(format!("{:?}", probe.vendor_id));
                            });
                            row.col(|ui| {
                                ui.label(format!("{:?}", probe.product_id));
                            });
                            row.col(|ui| {
                                ui.label(format!("{:?}", probe.serial_number));
                            });
                        });
                    }
                });
        });
    }
}