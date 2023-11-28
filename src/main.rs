#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::path::PathBuf;
use eframe::egui;
//use probe_rs::{MemoryInterface, Permissions, Session};

mod gdb_parser;
use gdb_parser::{GdbParser, VariableList};


fn main() -> Result<(), eframe::Error> {
    //let mut session = Session::auto_attach("STM32G431KBTx", Permissions::default());
    //let mut core = session.core(0);

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "STM32EguiMonitor",
        options,
        Box::new(|cc| {
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

            //let word = core.read_word_32(0x2000_0000)?;
            ui.label(format!("Hello '{}', age {}", self.name, self.age));

            ui.heading("ELF File Viewer");
            ui.horizontal(|ui| {
                ui.label("Path to ELF file:");
                ui.text_edit_singleline(&mut self.elf_path);
                if ui.button("Load").clicked() {
                    if let Ok(mut gdb_parser) = GdbParser::launch(&PathBuf::from(&self.elf_path)) {
                        if let Ok(variable_list) = gdb_parser.scan_variables() {
                            self.variable_list = variable_list.clone();
                            println!("file loaded");
                        }
                    }else{
                        println!("failed file load");
                    }
                }
            });
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
                ui.horizontal(|ui| {
                    ui.label("Symbol");
                    ui.label("Address");
                    ui.label("Type");
                });
                for vals in &self.search_results_list {
                    ui.horizontal(|ui| {
                        ui.label(&vals.name);
                        //ui.label(&format!("{:x}", &vals.address));
                        ui.label(&vals.address);
                        ui.label(&vals.types);
                    });
                }
                ui.separator();
        });
    }
}

