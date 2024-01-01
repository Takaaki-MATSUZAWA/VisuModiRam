use eframe::egui::{self, Button, Color32};
use egui_extras::{Column, TableBuilder};

use crate::debugging_tools::{GdbParser, ProbeInterface, VariableInfo};
use crate::monitor_ui::{self, *};

pub struct STM32EguiMonitor {
    my_probe: ProbeInterface,
    probe_setting_ui: ProbeSetting,
    symbol_serch_ui: SymbolSearch,
    probe_if_test_ui: ProbeIfTest,
    graph_test_ui: GraphTest,
    //widgets: Vec<monitor_ui::Widget<'static>>,
    watch_list: Vec<VariableInfo>,

    window_cnt: u32,
}

impl Default for STM32EguiMonitor {
    fn default() -> Self {
        let mut se = Self {
            my_probe: Default::default(),
            probe_setting_ui: Default::default(),
            symbol_serch_ui: Default::default(),
            probe_if_test_ui: ProbeIfTest::new(),
            graph_test_ui: GraphTest::new(),
            watch_list: Vec::new(),
            //widgets: Vec::new(),
            /*
            widgets: vec![
                Widget::new(0, "test 0".to_string(), Box::new(WidgetTest::new("aaa".to_string(), 42, self.watch_list))),
                Widget::new(1, "test 1".to_string(), Box::new(WidgetTest{name: "bbb".to_string(), age:12}))
            ],
             */
            window_cnt: 3,
        };
        //se.setup();
        se
    }
}

impl STM32EguiMonitor {
    #[cfg(disabke)]
    fn setup(&mut self) -> &mut Self {
        self.widgets.push(Widget::new(
            0,
            "test 0".to_string(),
            Box::new(WidgetTest::new("aaa".to_string(), 42)),
        ));
        self.widgets.push(Widget::new(
            1,
            "test 1".to_string(),
            Box::new(WidgetTest::new("bbb bbb ".to_string(), 12)),
        ));

        self
    }
}

impl eframe::App for STM32EguiMonitor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        #[cfg(disable)]
        for w in self.widgets.iter_mut() {
            w.set_watch_list_ptr(&self.watch_list);
        }

        egui::SidePanel::left("setting").show(ctx, |ui| {
            // add window test
            #[cfg(disable)]
            if ui.button("add window").clicked() {
                self.window_cnt += 1;
                self.widgets.push(Widget::new(
                    self.window_cnt,
                    format!("add window {}", self.window_cnt).to_string(),
                    Box::new(WidgetTest::new(
                        "bbb bbb ".to_string(),
                        self.window_cnt * 10,
                    )),
                ));
            }

            // Probe Setting
            //self.probe_setting_ui.ui(ui, _frame);

            ui.separator();

            // symbol search
            self.symbol_serch_ui.ui(ctx, ui, _frame);

            self.watch_list = Vec::new();
            for val in &mut self.symbol_serch_ui.selected_list {
                if val.is_selected {
                    self.watch_list.push(VariableInfo {
                        name: val.name.clone(),
                        types: val.types.clone(),
                        address: val.address.clone(),
                        size: 0,
                    });
                }
            }
            /*
            self.watch_list = self.symbol_serch_ui.selected_list.iter().map(|x| VariableInfo {
                name: x.name.clone(),
                types: x.types.clone(),
                address: x.address.clone(),
                size: 0,
            }).collect();
            */
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Centor Panel");
            // multi widget app test
            #[cfg(disable)]
            for wgt in self.widgets.iter_mut() {
                wgt.ui(ctx, ui);
            }
            //self.probe_if_test_ui.my_probe = Some(Box::new(self.my_probe.clone()));

            egui::Window::new(&self.probe_if_test_ui.name).show(ctx, |ui| {
                self.probe_if_test_ui.ui(&mut self.my_probe, ui, _frame);
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

// ----------------------------------------------------------------------------

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
