use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use env_logger::fmt::Color;
use std::path::PathBuf;

use crate::debugging_tools::{GdbParser, VariableInfo};

use rfd::FileDialog;

// ----------------------------------------------------------------------------
#[derive(Default, Clone)]
pub struct SelectableVariableInfo {
    pub name: String,
    pub types: String,
    pub address: String,
    pub size: usize,
    pub is_selected: bool,
}
// ----------------------------------------------------------------------------

#[derive(Default)]
pub struct SymbolSearch {
    input_elf_path: String,
    search_name: String,
    variable_list: Vec<VariableInfo>,
    pub selected_list: Vec<SelectableVariableInfo>,
    gdb_parser: Option<GdbParser>,
    target_mcu_id: String,
}

#[cfg(disable)]
impl Default for SymbolSearch {
    fn default() -> Self {
        Self {
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
    pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, _frame: &eframe::Frame) {
        ui.heading("ELF file loader");
        ui.horizontal(|ui| {
            ui.label("Path to ELF file :");
            ui.text_edit_singleline(&mut self.input_elf_path);

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("browse…").clicked() {
                if let Some(path) = FileDialog::new().pick_file() {
                    self.input_elf_path = path
                        .to_str()
                        .ok_or_else(|| "Failed to convert path to string")
                        .unwrap()
                        .to_string();
                }
            }

            let elf_path = format!("{}", shellexpand::tilde(&self.input_elf_path));
            let is_elf_file_exixt = std::path::Path::new(&elf_path).exists();

            if ui.button("Load").clicked() {
                if is_elf_file_exixt {
                    if let Some(mcu_id) =
                        crate::debugging_tools::search_target_mcu_name(&PathBuf::from(&elf_path))
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
                ui.label(RichText::new("ELF file is not found").color(Color32::RED));
            }
        });

        let mut prgres_text = "  Please load ELF file...";
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
                        self.selected_list.push(SelectableVariableInfo {
                            name: vals.name,
                            types: vals.types,
                            address: vals.address,
                            size: vals.size,
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
        ui.heading("Infomation");
        ui.label("Project Name : ");
        ui.horizontal(|ui| {
            ui.label("Target MUC : ");
            ui.text_edit_singleline(&mut self.target_mcu_id);
        });

        ui.separator();
        ui.heading("Variable list");
        ui.horizontal(|ui| {
            ui.label("filler or variable name");
            ui.text_edit_singleline(&mut self.search_name);
        });

        let window_width = ctx.available_rect().width() / 2.0;

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .vscroll(true)
            .drag_to_scroll(true)
            //.max_scroll_height(10.)
            .column(Column::initial(10.).resizable(true))
            .column(Column::initial(120.).resizable(true))
            .column(Column::initial(160.).resizable(true))
            //.column(Column::auto().resizable(true).clip(true))
            .column(
                Column::initial(window_width - (10. + 120. + 160. + 50.))
                    .at_least(50.0)
                    .resizable(true),
            )
            .header(9.0, |mut header| {
                header.col(|ui| {
                    //ui.set_width(10.);
                });
                header.col(|ui| {
                    ui.heading("Address");
                    //ui.set_width(120.0);
                });
                header.col(|ui| {
                    ui.heading("Type");
                    //ui.set_width(160.0);
                });
                header.col(|ui| {
                    //ui.clip_rect();
                    //ui.set_width(100.0);
                    let window_width = ctx.available_rect().width() / 2.0;
                    //ui.set_width(window_width - (10.+120.+160.+50.));
                    ui.heading("Symbol Name");
                });
            })
            .body(|mut body| {
                for selected in self.selected_list.iter_mut() {
                    if selected
                        .name
                        .to_lowercase()
                        .contains(&self.search_name.to_lowercase())
                    {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut selected.is_selected, "")
                                    .on_hover_text("add watch list");
                            });
                            row.col(|ui| {
                                ui.label(&selected.address);
                            });
                            row.col(|ui| {
                                ui.label(&selected.types);
                            });
                            row.col(|ui| {
                                ui.label(&selected.name).on_hover_text(&selected.name);
                            });
                        });
                    }
                }
            });
    }
}
