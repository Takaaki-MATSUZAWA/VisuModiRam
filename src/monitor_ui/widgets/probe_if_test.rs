use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::debugging_tools::ProbeInterface;

pub struct ProbeIfTest {
    pub name: String,
    pub my_probe: Option<Box<ProbeInterface>>, // Boxを使用して所有権を保持
}

impl ProbeIfTest {
    pub fn new() -> Self {
        Self {
            name: "probe interface test".to_string(),
            my_probe: None,
        }
    }

    pub fn ui(&mut self, probe_if: &mut ProbeInterface, ui: &mut egui::Ui, frame: &eframe::Frame) {
        ui.heading("probe interface test");
        ui.horizontal(|ui| {
            ui.label("waching");

            if ui.button("start").clicked() {
                probe_if.watching_start(std::time::Duration::from_millis(100));
                /*
                if let Some(probe) = &mut self.my_probe {
                    probe.watching_start(std::time::Duration::from_millis(100));
                }
                 */
            }

            if ui.button("stop").clicked() {
                probe_if.watching_stop();
                /*}
                if let Some(probe) = &mut self.my_probe {
                    probe.watching_stop();
                }
                 */
            }
        });

        #[cfg(disable)]
        ui.label(format!("data: {}", probe_if.get_data()));
        /*
        if let Some(probe) = &mut self.my_probe {
            ui.label(format!("data: {}", probe.get_data()));
        }
         */
    }
}
