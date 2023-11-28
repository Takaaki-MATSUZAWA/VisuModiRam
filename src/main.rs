#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::path::PathBuf;
use eframe::egui;
use egui_extras::{TableBuilder, Column};

mod gdb_parser;
use gdb_parser::{GdbParser, VariableList};


fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 480.0)),
        ..Default::default()
    };
    eframe::run_native(
        "STM32EguiMonitor",
        options,
        Box::new(|_cc| {
            Box::<STM32EguiMonitor>::default()
        }),
    )
}

struct STM32EguiMonitor {
    name: String,
    age: u32,
    elf_path: String,
    search_name: String,
    variable_list: Vec<VariableList>,
    search_results_list: Vec<VariableList>,
    gdb_parser: Option<GdbParser>,

}

impl Default for STM32EguiMonitor {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            elf_path: Default::default(),
            search_name: Default::default(),
            variable_list: Vec::new(),
            search_results_list: Vec::new(),
            gdb_parser: None,
        }
    }
}

impl eframe::App for STM32EguiMonitor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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

            ui.heading("ELF File Viewer");
            ui.horizontal(|ui| {
                ui.label("Path to ELF file:");
                ui.text_edit_singleline(&mut self.elf_path);
                if ui.button("Load").clicked() {
                    if let Ok(gdb_parser) = GdbParser::launch(&PathBuf::from(&self.elf_path)) {
                        self.variable_list = Vec::new();

                        self.gdb_parser = Some(gdb_parser);
                        if let Some(gdb_parser) = &mut self.gdb_parser {
                            gdb_parser.scan_variables_none_blocking_start();
                        }
                        /*
                        if let Ok(variable_list) = gdb_parser.scan_variables() {
                            self.variable_list = variable_list.clone();
                            println!("file loaded");
                        }
                        */
                    }else{
                        println!("failed file load");
                    }
                }
            });
            
            if let Some(gdb_parser) = &mut self.gdb_parser {
                let _now_prgress = gdb_parser.get_scan_progress();

                if _now_prgress <1.0{
                    ui.add(egui::ProgressBar::new(gdb_parser.get_scan_progress()).text("Loading..."));
                }else{
                    ui.add(egui::ProgressBar::new(1.0).text("complete"));
                    if self.variable_list.is_empty(){
                        self.variable_list = gdb_parser.load_variable_list();
                    }
                }
            }

            if ui.text_edit_singleline(&mut self.search_name)
                //.hint_text("Enter variable name...")
                .changed()
                {
                    self.search_results_list = Vec::new();

                    for vals in &self.variable_list{
                        if vals.name.to_lowercase().contains(&self.search_name.to_lowercase()){
                            self.search_results_list.push(vals.clone());
                        }
                    }
                }

            ui.separator();
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
                        ui.set_width(300.0);
                    });
                })
                .body(|mut body| {
                    for vals in &self.search_results_list {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.label(&vals.address);
                            });
                            row.col(|ui| {
                                ui.label(&vals.types);
                            });
                            row.col(|ui| {
                                ui.label(&vals.name);
                            });
                        });
                    }
                });
        });
    }
}
