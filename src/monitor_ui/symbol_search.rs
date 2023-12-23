use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;

use crate::debugging_tools::{GdbParser, VariableInfo};

pub struct SymbolSearch{
    input_elf_path: String,
    search_name: String,
    variable_list: Vec<VariableInfo>,
    pub selected_list: Vec<MainVariableInfo>,
    gdb_parser: Option<GdbParser>,
    target_mcu_id: String,
}

pub struct MainVariableInfo {
    pub name: String,
    pub types: String,
    pub address: String,
    pub is_selected: bool,
}

impl Default for SymbolSearch {
    fn default() -> Self {
        Self{
            input_elf_path: Default::default(),
            search_name: Default::default(),
            variable_list: Vec::new(),
            selected_list: Vec::new(),
            gdb_parser: None,
            target_mcu_id: Default::default(),
        }
    }
}

impl SymbolSearch {
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame) {
        ui.horizontal(|ui| {
            ui.label("target MUC:");
            ui.text_edit_singleline(&mut self.target_mcu_id);
        });

        ui.horizontal(|ui| {
            ui.label("Path to ELF file:");
            ui.text_edit_singleline(&mut self.input_elf_path);

            let elf_path = format!("{}", shellexpand::tilde(&self.input_elf_path));
            let is_elf_file_exixt = std::path::Path::new(&elf_path).exists();

            if ui.button("Load").clicked() {
                if is_elf_file_exixt {
                    if let Some(mcu_id) = crate::debugging_tools::search_target_mcu_name(&PathBuf::from(&elf_path))
                    {
                        self.target_mcu_id = mcu_id;
                    }

                    if let Ok(gdb_parser) = GdbParser::launch(&PathBuf::from(&elf_path)) {
                        self.variable_list = Vec::new();

                        self.gdb_parser = Some(gdb_parser);
                        if let Some(gdb_parser) = &mut self.gdb_parser {
                            gdb_parser.scan_variables_none_blocking_start();
                            println!("scan start");
                        }
                    } else {
                        println!("failed file load");
                    }
                }
            }

            if !is_elf_file_exixt {
                ui.label("ELF file is not found");
            }
        });

        let mut prgres_text = "";
        let mut now_progress = 0.0;
        let mut prgress_anime = false;

        if let Some(gdb_parser) = &mut self.gdb_parser {
            now_progress = gdb_parser.get_scan_progress();
            prgress_anime = true;

            if now_progress < 1.0 {
                prgres_text = "Loading...";
            } else {
                prgres_text = "complete";
                prgress_anime = false;

                if self.variable_list.is_empty() {
                    self.variable_list = gdb_parser.load_variable_list();

                    self.selected_list = Vec::new();
                    for vals in self.variable_list.clone() {
                        self.selected_list.push(MainVariableInfo {
                            name: vals.name,
                            types: vals.types,
                            address: vals.address,
                            is_selected: false,
                        });
                    }
                }
            }
        }

        ui.add(
            egui::ProgressBar::new(now_progress)
                .text(prgres_text)
                .animate(prgress_anime),
        );

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Search variable name");
            ui.text_edit_singleline(&mut self.search_name);
        });

        ui.push_id(1, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .vscroll(true)
                .drag_to_scroll(true)
                .max_scroll_height(10.)
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true).clip(true))
                .header(9.0, |mut header| {
                    header.col(|ui| ui.set_width(10.));

                    header.col(|ui| {
                        ui.heading("Address");
                        ui.set_width(80.0);
                    });
                    header.col(|ui| {
                        ui.heading("Type");
                        ui.set_width(100.0);
                    });
                    header.col(|ui| {
                        ui.clip_rect();
                        ui.heading("Symbol");
                    });
                })
                .body(|mut body| {
                    for selected in self.selected_list.iter_mut() {
                        if selected.name.to_lowercase().contains(&self.search_name.to_lowercase()) {
                            body.row(18.0, |mut row| {
                                row.col(|ui| {
                                    ui.checkbox(&mut selected.is_selected, "");
                                });
                                row.col(|ui| {
                                    ui.label(&selected.address);
                                });
                                row.col(|ui| {
                                    ui.label(&selected.types);
                                });
                                row.col(|ui| {
                                    ui.label(&selected.name)
                                        .on_hover_text(&selected.name);
                                });
                            });
                        }
                    }
                });
        });
    }
}