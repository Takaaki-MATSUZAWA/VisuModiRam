use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;

use crate::debugging_tools::{GdbParser, VariableList};

pub struct SymbolSearch{
    input_elf_path: String,
    elf_path: String,
    search_name: String,
    variable_list: Vec<VariableList>,
    search_results_list: Vec<VariableList>,
    pub selected_list: Vec<MainVariableList>,
    gdb_parser: Option<GdbParser>,
    elf_file_is_not_found: bool,
    target_mcu_id: String,
}

pub struct MainVariableList {
    pub name: String,
    pub types: String,
    pub address: String,
    pub is_selected: bool,
}

impl Default for SymbolSearch {
    fn default() -> Self {
        Self{
            input_elf_path: Default::default(),
            elf_path: Default::default(),
            search_name: Default::default(),
            variable_list: Vec::new(),
            search_results_list: Vec::new(),
            selected_list: Vec::new(),
            gdb_parser: None,
            elf_file_is_not_found: false,
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
            self.elf_path = format!("{}", shellexpand::tilde(&self.input_elf_path));

            if ui.button("Load").clicked() {
                if !std::path::Path::new(&self.elf_path).exists() {
                    self.elf_file_is_not_found = true;
                } else {
                    self.elf_file_is_not_found = false;

                    if let Some(mcu_id) = crate::debugging_tools::search_target_mcu_name(&PathBuf::from(&self.elf_path))
                    {
                        self.target_mcu_id = mcu_id;
                    }

                    if let Ok(gdb_parser) = GdbParser::launch(&PathBuf::from(&self.elf_path)) {
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
            if self.elf_file_is_not_found {
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
                    self.search_results_list = self.variable_list.clone();

                    self.selected_list = Vec::new();
                    for vals in self.variable_list.clone() {
                        self.selected_list.push(MainVariableList {
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
            if ui
                .text_edit_singleline(&mut self.search_name)
                //.hint_text("Enter variable name...")
                .changed()
            {
                self.search_results_list = Vec::new();

                for vals in &self.variable_list {
                    if vals
                        .name
                        .to_lowercase()
                        .contains(&self.search_name.to_lowercase())
                    {
                        self.search_results_list.push(vals.clone());
                    }
                }
            }
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
                    //for vals in &self.search_results_list {
                    for i in 0..self.selected_list.len() {
                        body.row(18.0, |mut row| {
                            if self.selected_list[i]
                                .name
                                .to_lowercase()
                                .contains(&self.search_name.to_lowercase())
                            {
                                row.col(|ui| {
                                    ui.checkbox(&mut self.selected_list[i].is_selected, "");
                                });
                                row.col(|ui| {
                                    ui.label(&self.selected_list[i].address);
                                });
                                row.col(|ui| {
                                    ui.label(&self.selected_list[i].types);
                                });
                                row.col(|ui| {
                                    ui.label(&self.selected_list[i].name)
                                        .on_hover_text(&self.selected_list[i].name);
                                });
                            }
                        });
                    }
                });
        });
    }
}