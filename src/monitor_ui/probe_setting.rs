use eframe::egui;
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

use crate::debugging_tools::ProbeInterface;

// ----------------------------------------------------------------------------
struct SelectableProbeInfo {
    is_selected: bool,
    info: probe_rs::DebugProbeInfo,
}
// ----------------------------------------------------------------------------

pub struct ProbeSetting {
    probes: Vec<SelectableProbeInfo>,
    probe_if: ProbeInterface,
    select_sn: Option<String>,
}

impl Default for ProbeSetting {
    fn default() -> Self {
        Self {
            probes: Vec::new(),
            probe_if: Default::default(),
            select_sn: None,
        }
    }
}
impl ProbeSetting {
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.heading("Debug probe select");

        if ui.add(egui::Button::new("probe check")).clicked() {
            let cn_probes = self.probe_if.get_connected_probes();
            self.probes = cn_probes
                .iter()
                .map(|prb| SelectableProbeInfo {
                    is_selected: false,
                    info: prb.clone(),
                })
                .collect();
        }

        StripBuilder::new(ui)
            .size(Size::remainder().at_least(100.0)) // for the table
            .size(Size::exact(500.)) // for the source code link
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        //
                        TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .max_scroll_height(5.)
                            .vscroll(true)
                            .drag_to_scroll(true)
                            .column(Column::auto().resizable(true))
                            .column(Column::auto().resizable(true))
                            .column(Column::auto().resizable(true))
                            .column(Column::auto().resizable(true))
                            .column(Column::auto().resizable(true))
                            .column(Column::auto().resizable(true))
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.set_width(10.);
                                });
                                header.col(|ui| {
                                    ui.set_width(60.);
                                    ui.heading("name");
                                });

                                header.col(|ui| {
                                    ui.set_width(60.);
                                    ui.heading("type");
                                });
                                header.col(|ui| {
                                    ui.set_width(70.);
                                    ui.heading("vnd_id");
                                });
                                header.col(|ui| {
                                    ui.set_width(70.);
                                    ui.heading("prd_id");
                                });
                                header.col(|ui| {
                                    ui.set_width(260.);
                                    ui.heading("Serial number");
                                });
                            })
                            .body(|mut body| {
                                for probe in self.probes.iter_mut() {
                                    body.row(18.0, |mut row| {
                                        row.col(|ui| {
                                            ui.radio_value(
                                                &mut self.select_sn,
                                                probe.info.serial_number.clone(),
                                                "",
                                            );
                                        });
                                        row.col(|ui| {
                                            ui.label(&probe.info.identifier);
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{:?}", probe.info.probe_type));
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{:?}", probe.info.vendor_id));
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{:?}", probe.info.product_id));
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{:?}", probe.info.serial_number));
                                        });
                                    });
                                }
                            });
                    });
                });
                strip.cell(|ui| {
                    ui.separator();
                    ui.label("new contents");
                    ui.label(format!("select prove --> {:?}", self.select_sn));
                });
            });
    }
}
