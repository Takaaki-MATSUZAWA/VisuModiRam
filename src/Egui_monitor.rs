use eframe::egui::{self, Button, Color32};
use egui_extras::{Column, TableBuilder};

use crate::debugging_tools::{GdbParser, ProbeInterface, VariableList};
use crate::monitor_ui::{ProbeSetting, SymbolSearch, ProbeIfTest, GraphTest};

 #[cfg(test)]
mod tests {
    use std::io::Write;

    // > cargo test --all -- --nocapture test_search_target_mcu_name
    use super::*;
    use std::path::PathBuf;

    const ELF_FILE: &str = r"~\Dropbox\project\STM32Cube\VScode_workspace\STSPIN32G4_mobility2022_motor\build\debug\build\STSPIN32G4_mobility2022_motor.elf";

    #[test]
    fn test_search_target_mcu_name() {
        let path = PathBuf::from(format!("{}", shellexpand::tilde(&ELF_FILE.to_string())));
        let result = crate::debugging_tools::search_target_mcu_name(&path);

        assert!(result.is_some());
        let unwrapped_result = result.unwrap();

        println!("{}", unwrapped_result);
        assert_eq!(unwrapped_result, "STM32G431VBTx");
    }

    #[test]
    fn test_get_symbol_size() {
        let elf_path = format!("{}", shellexpand::tilde(&ELF_FILE.to_string()));
        let mut gdb_parser = GdbParser::launch(&PathBuf::from(&elf_path)).unwrap();

        gdb_parser.scan_variables_none_blocking_start();
        println!("scan start");

        loop {
            let now_progress = gdb_parser.get_scan_progress();
            print!("\rprocess {:3} %", (now_progress * 100.0) as i32);
            std::io::stdout().flush();
            std::thread::sleep(std::time::Duration::from_millis(100));

            if now_progress == 1.0 {
                break;
            }
        }

        println!("\nscan complete");

        let variable_list = gdb_parser.load_variable_list();

        for num in 0..10 {
            println!(
                "add:{}, type:{}, size:{:?}, name:{}",
                variable_list[num].address,
                variable_list[num].types,
                variable_list[num].size,
                variable_list[num].name
            );
        }
    }
}

pub struct STM32EguiMonitor {
    name: String,
    age: u32,
    time: f64,
    my_probe: ProbeInterface,
    probe_setting_ui: ProbeSetting,
    symbol_serch_ui: SymbolSearch,
    probe_if_test_ui: ProbeIfTest,
    graph_test_ui: GraphTest,
}

struct MainVariableList {
    name: String,
    types: String,
    address: String,
    is_selected: bool,
}

impl Default for STM32EguiMonitor {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            time: 0.0,
            my_probe: Default::default(),
            probe_setting_ui: Default::default(),
            symbol_serch_ui: Default::default(),
            probe_if_test_ui: ProbeIfTest::new(),
            graph_test_ui: GraphTest::new(),
        }
    }
}

impl eframe::App for STM32EguiMonitor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::SidePanel::left("setting").show(ctx, |ui| {
            // Probe Setting
            self.probe_setting_ui.ui(ui, _frame);

            ui.separator();

            // symbol search
            self.symbol_serch_ui.ui(ui, _frame);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Centor Panel");

            //self.probe_if_test_ui.my_probe = Some(Box::new(self.my_probe.clone()));

            egui::Window::new(&self.probe_if_test_ui.name).show(ctx, |ui| {
                self.probe_if_test_ui.ui(&mut self.my_probe, ui, _frame);
            });

            egui::Window::new("Window").show(ctx, |ui| {
                ui.heading("STM32EguiMonitor");
                ui.horizontal(|ui| {
                    let name_label = ui.label("Your name: ");
                    ui.text_edit_singleline(&mut self.name)
                        .labelled_by(name_label.id);
                });
                ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
                if ui.button("Click each year").clicked() {
                    self.age += 1;
                }

                ui.label(format!("Hello '{}', age {}", self.name, self.age));
            });

            egui::Window::new("Graph Window")
                .default_size(egui::vec2(200.0, 200.0)) // ウィンドウのデフォルトサイズを設定
                .show(ctx, |ui| {
                    self.graph_test_ui.ui(&mut self.my_probe, ui, ctx, _frame);
                });

            // watch valiables list
            egui::Window::new("watch valiables list").show(ctx, |ui| {
                ui.heading("watch valiables list");
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .vscroll(true)
                    .column(Column::auto().resizable(true))
                    .column(Column::auto().resizable(true))
                    .column(Column::auto().resizable(true))
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Address");
                            ui.set_width(80.0);
                        });
                        header.col(|ui| {
                            ui.heading("Type");
                            ui.set_width(100.0);
                        });
                        header.col(|ui| {
                            ui.heading("Symbol");
                        });
                    })
                    .body(|mut body| {
                        for vals in &self.symbol_serch_ui.selected_list {
                            body.row(18.0, |mut row| {
                                if vals.is_selected {
                                    row.col(|ui| {
                                        ui.label(&vals.address);
                                    });
                                    row.col(|ui| {
                                        ui.label(&vals.types);
                                    });
                                    row.col(|ui| {
                                        ui.label(&vals.name);
                                    });
                                }
                            });
                        }
                    });
            });
        });
    }
}
