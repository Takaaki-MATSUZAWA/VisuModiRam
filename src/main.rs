#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use goblin::elf::{Elf, sym::Sym};
use goblin::Object;
use std::fs;
use eframe::egui;
//use probe_rs::{MemoryInterface, Permissions, Session};

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
    elf_file: Vec<u8>,
    search_name: String,
    address: Option<u64>,
    search_results: Vec<String>,
}

impl Default for STM32EguiMonitor {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            elf_path: Default::default(),
            elf_file: Default::default(),
            search_name: Default::default(),
            address: None,
            search_results: Vec::new(),
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
                    self.address = None;
                    if let Ok(buffer) = fs::read(&self.elf_path) {
                        match Object::parse(&buffer) {
                            Ok(Object::Elf(elf)) => {
                                self.elf_file = buffer.to_vec();
                                print!("file loaded")
                                //println!("elf: {:#?}", &elf);
                            },
                            Ok(Object::PE(pe)) => {
                                //println!("pe: {:#?}", &pe);
                            },
                            Ok(Object::Mach(mach)) => {
                                //println!("mach: {:#?}", &mach);  
                            },
                            Ok(Object::Archive(archive)) => {
                                //println!("archive: {:#?}", &archive);
                            },
                            Ok(Object::Unknown(magic)) => { println!("unknown magic: {:#x}", magic) },
                            Err(_) => {
                                print!("parse error");
                            }
                        }
                    }else{
                        print!("read false");
                    }
                }
            });
            if ui.text_edit_singleline(&mut self.search_name)
                //.hint_text("Enter variable name...")
                .changed()
                {
                    self.search_results = Vec::new();
                    match Object::parse(&(self.elf_file)) {
                        Ok(Object::Elf(elf)) => {
                            //println!("elf: {:#?}", &elf);
                            let symbols = elf.syms.to_vec();
                            let sym_names = elf.strtab;
                            for name in symbols{
                                if sym_names[name.st_name].contains(&self.search_name){
                                    self.search_results.push(format!("elf: {:#?}", &sym_names[name.st_name]));
                                    self.search_results.push(format!("elf: {:#?}", &name));
                                }
                            }
                            //println!("elf: {:#?}", &sym_names[(name.st_name)]);
                        },
                        Ok(Object::PE(pe)) => {
                            //println!("pe: {:#?}", &pe);
                        },
                        Ok(Object::Mach(mach)) => {
                            //println!("mach: {:#?}", &mach);
                        },
                        Ok(Object::Archive(archive)) => {
                            //println!("archive: {:#?}", &archive);
                        },
                        Ok(Object::Unknown(magic)) => { println!("unknown magic: {:#x}", magic) },
                        Err(_) => {
                            print!("parse error");
                        }
                    }
                }
            if ui.button("Search").clicked() {
                    match Object::parse(&(self.elf_file)) {
                        Ok(Object::Elf(elf)) => {
                            //println!("elf: {:#?}", &elf);
                            let symbols = elf.syms.to_vec();
                            let sym_names = elf.strtab;
                            for name in symbols{
                                if sym_names[name.st_name].contains(&self.search_name){
                                    println!("elf: {:#?}", &sym_names[name.st_name]);
                                    println!("elf: {:#?}", &name);
                                }
                            }
                            //println!("elf: {:#?}", &sym_names[(name.st_name)]);
                        },
                        Ok(Object::PE(pe)) => {
                            //println!("pe: {:#?}", &pe);
                        },
                        Ok(Object::Mach(mach)) => {
                            //println!("mach: {:#?}", &mach);
                        },
                        Ok(Object::Archive(archive)) => {
                            //println!("archive: {:#?}", &archive);
                        },
                        Ok(Object::Unknown(magic)) => { println!("unknown magic: {:#x}", magic) },
                        Err(_) => {
                            print!("parse error");
                        }
                    }
                }

            for rsrt in &self.search_results {
                ui.label(rsrt);
            }
            if let Some(address) = self.address {
                ui.label(format!("Address: 0x{:x}", address));
            } else {
                ui.label("Address: N/A");
            }
        });
    }
}
