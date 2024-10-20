use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

use rfd::FileDialog;
use std::path::PathBuf;

use crate::debugging_tools::*;
use probe_rs::Probe;
use regex::Regex;

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct MemInfo {
    size: f64,
    used: f64,
    percent: f64,
}

impl MemInfo {
    pub fn set_used_size(&mut self, used: f64) {
        self.used = used;
        self.percent = (self.used / self.size) * 100.0;
    }

    pub fn calc_percent(&mut self) {
        self.percent = (self.used / self.size) * 100.0;
    }
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct TargetMCUInfo {
    id: String,
    id_rock: bool,
    id_not_found: bool,
    rom: MemInfo,
    ram: MemInfo,
    candidate_list: Vec<String>,
}

use probe_rs::config::{get_target_by_name, search_chips, MemoryRegion};

impl TargetMCUInfo {
    pub fn check_id(&mut self, id: &str) {
        if let Ok(chip) = get_target_by_name(id) {
            self.id = chip.name.clone();
            self.id_not_found = false;
            if let Some((ram_size, nvm_size)) = Self::get_memory_sizes(&chip.name) {
                self.rom.size = nvm_size as f64;
                self.ram.size = ram_size as f64;
                self.rom.calc_percent();
                self.ram.calc_percent();
            }
        } else {
            self.id_not_found = true;
            // idを検索し、結果が空の場合は後ろから1文字ずつ削って再検索
            let mut search_id = id.to_string();
            while search_id.len() > 0 {
                if let Ok(chips) = search_chips(&search_id) {
                    if !chips.is_empty() {
                        self.candidate_list = chips
                            .into_iter()
                            .map(|chip| {
                                let (ram, rom) = Self::get_memory_sizes(&chip).unwrap_or((0, 0));
                                format!("{:<10} (RAM: {:>3}KB, ROM: {:>3}KB)", chip, ram, rom)
                            })
                            .collect();
                        break;
                    }
                }
                search_id.pop();
            }

            // 候補が見つからなかった場合
            if self.candidate_list.is_empty() {
                self.id_not_found = true;
            }
        }
    }

    fn get_memory_sizes(chip_name: &str) -> Option<(u32, u32)> {
        if let Ok(chip) = get_target_by_name(chip_name) {
            let ram_size = chip
                .memory_map
                .iter()
                .filter_map(|region| {
                    if let MemoryRegion::Ram(ram) = region {
                        Some(ram.range.end - ram.range.start)
                    } else {
                        None
                    }
                })
                .map(|size| size as f64) // 各要素をf64に変換
                .sum::<f64>();

            let nvm_size = chip
                .memory_map
                .iter()
                .filter_map(|region| {
                    if let MemoryRegion::Nvm(nvm) = region {
                        Some(nvm.range.end - nvm.range.start)
                    } else {
                        None
                    }
                })
                .map(|size| size as f64) // 各要素をf64に変換
                .sum::<f64>();

            Some(((ram_size / 1024.0) as u32, (nvm_size / 1024.0) as u32))
        } else {
            None
        }
    }
}

// ----------------------------------------------------------------------------
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct SymbolSearch {
    input_elf_path: String,
    search_name: String,
    variable_list: Vec<VariableInfo>,
    selected_list: Vec<SelectableVariableInfo>,
    expand_list_flag: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    elf_parser: Option<ELFParser>,
    project_name: String,

    target_mcu: TargetMCUInfo,
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct ProbeSetting {
    #[cfg_attr(feature = "serde", serde(skip))]
    probes: Vec<probe_rs::DebugProbeInfo>,
    select_sn: Option<String>,

    #[cfg_attr(feature = "serde", serde(skip))]
    flash_probe_if: ProbeInterface,
}

// ----------------------------------------------------------------------------
#[derive(PartialEq, Clone, Copy)]
enum ProgresState {
    None,
    SymbolSearching,
    SymbolSearchComplite,
    ElfFlashErasing,
    ElfFlashWriteing,
    ElfFlashComplite,
    ElfFlashFaild,
}

impl Default for ProgresState {
    fn default() -> Self {
        Self::None
    }
}
// ----------------------------------------------------------------------------
#[derive(Default)]
struct PrgressBarContext {
    state: ProgresState,
    progress: f32,
}

impl PrgressBarContext {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        use ProgresState::*;
        let text = match self.state {
            None => "  Please load ELF file...".to_string(),
            SymbolSearching => "Symbol Search: Loading...".to_string(),
            SymbolSearchComplite => "Symbol Search: complete".to_string(),
            ElfFlashErasing => "Flash: Erasing...".to_string(),
            ElfFlashWriteing => "Flash: Programing...".to_string(),
            ElfFlashComplite => "Flash: Complete!".to_string(),
            ElfFlashFaild => "Flash: Failed !!!".to_string(),
        };

        let animete = self.state == SymbolSearching
            || self.state == ElfFlashErasing
            || self.state == ElfFlashWriteing;

        ui.add(
            egui::ProgressBar::new(self.progress)
                .text(text.as_str())
                .animate(animete),
        );
    }

    pub fn set_progress(&mut self, state: ProgresState, progress: f32) {
        self.state = state;
        self.progress = progress;
    }

    pub fn complete(&mut self, next_state: ProgresState) {
        use ProgresState::*;
        let tmp_state = self.state.clone();

        if tmp_state == SymbolSearching && next_state == SymbolSearchComplite {
            self.state = SymbolSearchComplite;
            self.progress = 1.0;
        }

        if tmp_state == ElfFlashWriteing && next_state == ElfFlashComplite {
            self.state = ElfFlashComplite;
            self.progress = 1.0;
        }
    }

    pub fn faild(&mut self) {
        self.state = ProgresState::ElfFlashFaild;
    }
}
// ----------------------------------------------------------------------------
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SettingTab {
    symbol_search: SymbolSearch,
    probe_setting: ProbeSetting,

    #[cfg_attr(feature = "serde", serde(skip))]
    progress_bar: PrgressBarContext,
}

impl eframe::App for SettingTab {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let window_width = ctx.available_rect().width();

        egui::SidePanel::left("elf parser")
            //.resizable(true)
            .exact_width(window_width / 2.0)
            .show(ctx, |ui| {
                self.symbol_search_ui(ctx, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.probe_setting_ui(ui);
        });
    }
}

// ----------------------------------------------------------------------------
impl SettingTab {
    fn symbol_search_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let mut download_enable = false;
        ui.heading("ELF file loader");
        ui.horizontal(|ui| {
            ui.label("Path to ELF file :");
            ui.text_edit_singleline(&mut self.symbol_search.input_elf_path);

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("browse…").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("ELF file", &["elf"])
                    .pick_file()
                {
                    self.symbol_search.input_elf_path = path
                        .to_str()
                        .ok_or_else(|| "Failed to convert path to string")
                        .unwrap()
                        .to_string();
                }
            }

            let elf_path = format!("{}", shellexpand::tilde(&self.symbol_search.input_elf_path));
            let is_elf_file_exixt = std::path::Path::new(&elf_path).exists();
            let is_not_elf_file = !elf_path.ends_with(".elf");

            if ui.button("Load").clicked() {
                self.check_probe();
                if is_elf_file_exixt && !is_not_elf_file {
                    if let Some(mcu_id) =
                        crate::debugging_tools::search_target_mcu_name(&PathBuf::from(&elf_path))
                    {
                        if !self.symbol_search.target_mcu.id_rock {
                            self.symbol_search.target_mcu.id = mcu_id.clone();
                            self.symbol_search.target_mcu.check_id(&mcu_id);
                        }
                    } else {
                        self.symbol_search.target_mcu.id = "".to_string();
                    }

                    if let Ok(elf_parser) = ELFParser::launch(&PathBuf::from(&elf_path)) {
                        self.symbol_search.variable_list = Vec::new();
                        #[cfg(debug_assertions)]
                        println!("scaner launched");

                        self.symbol_search.elf_parser = Some(elf_parser);
                        if let Some(elf_parser) = &mut self.symbol_search.elf_parser {
                            elf_parser.scan_variables_none_blocking_start();
                            #[cfg(debug_assertions)]
                            println!("scan start");
                        }

                        let path_parts: Vec<&str> = elf_path.split("\\").collect();
                        let file_name = path_parts.last().unwrap_or(&"");
                        let path_parts: Vec<&str> = file_name.split("/").collect();
                        let file_name = path_parts.last().unwrap_or(&"");
                        self.symbol_search.project_name = file_name.to_string();
                        self.symbol_search.project_name = self
                            .symbol_search
                            .project_name
                            .trim_end_matches(".elf")
                            .to_string();
                    } else {
                        #[cfg(debug_assertions)]
                        println!("failed file load");
                    }
                }
            }

            if !is_elf_file_exixt {
                ui.label(RichText::new("ELF file is not found").color(Color32::RED));
            } else if is_not_elf_file {
                ui.label(RichText::new("Not ELF file").color(Color32::RED));
            } else {
                if self.probe_setting.probes.len() > 0 {
                    download_enable = true;
                }
            }
        });

        if ui
            .add_enabled(download_enable, egui::Button::new("Download"))
            .clicked()
        {
            if self.probe_setting.probes.len() > 0 {
                let elf_path =
                    format!("{}", shellexpand::tilde(&self.symbol_search.input_elf_path));

                let setting = self.get_watch_setting();
                let _ = self.probe_setting.flash_probe_if.set_probe(setting);
                self.probe_setting
                    .flash_probe_if
                    .flash(PathBuf::from(&elf_path));
            }
        }

        if let Some(elf_parser) = &mut self.symbol_search.elf_parser {
            let now_progress = elf_parser.get_scan_progress();

            if now_progress < 1.0 {
                ctx.request_repaint();
                self.progress_bar
                    .set_progress(ProgresState::SymbolSearching, now_progress);
            } else {
                if self.symbol_search.variable_list.is_empty() {
                    self.symbol_search.variable_list = elf_parser.load_variable_list();
                    if !self.symbol_search.variable_list.is_empty() {
                        ctx.request_repaint();
                        SelectableVariableInfo::fetch(
                            &self.symbol_search.variable_list,
                            &mut self.symbol_search.selected_list,
                        );

                        self.progress_bar
                            .complete(ProgresState::SymbolSearchComplite);

                        let elf_path =
                            format!("{}", shellexpand::tilde(&self.symbol_search.input_elf_path));
                        if let Some(res) = Self::get_memory_usage(&PathBuf::from(&elf_path)) {
                            self.symbol_search.target_mcu.rom.set_used_size(res.0);
                            self.symbol_search.target_mcu.ram.set_used_size(res.1);
                        } else {
                            self.symbol_search.target_mcu.rom.set_used_size(0.0);
                            self.symbol_search.target_mcu.ram.set_used_size(0.0);
                        }
                    }
                }
            }
        }

        if self.probe_setting.flash_probe_if.get_flash_progress().state != FlashProgressState::None
        {
            let flash_progress = self.probe_setting.flash_probe_if.get_flash_progress();

            let mut repaint_flag = flash_progress.progress < 1.0;
            if (flash_progress.progress == 1.0)
                && (self.progress_bar.state != ProgresState::ElfFlashComplite)
            {
                repaint_flag = true;
            }

            use FlashProgressState::*;
            match flash_progress.state {
                None => {}
                Erasing => {
                    self.progress_bar.set_progress(
                        ProgresState::ElfFlashErasing,
                        flash_progress.progress as f32,
                    );
                }
                Programing => {
                    self.progress_bar.set_progress(
                        ProgresState::ElfFlashWriteing,
                        flash_progress.progress as f32,
                    );
                }
                Finished => {
                    self.progress_bar.complete(ProgresState::ElfFlashComplite);
                }
                Failed => {
                    self.progress_bar.faild();
                }
            };

            if repaint_flag {
                ctx.request_repaint();
            }
        }

        self.progress_bar.show(ui);

        ui.separator();
        ui.heading("Infomation");
        ui.label(format!(
            "Project Name : {}",
            self.symbol_search.project_name
        ));
        ui.horizontal(|ui| {
            ui.label("Target MCU :");
            if ui
                .text_edit_singleline(&mut self.symbol_search.target_mcu.id)
                .on_hover_ui(|ui| {
                    if self.symbol_search.target_mcu.id_not_found {
                        ui.label("Candidate List");
                        for candidate in &self.symbol_search.target_mcu.candidate_list {
                            ui.label(format!("{}", candidate));
                        }
                    }
                })
                .changed()
            {
                let id = self.symbol_search.target_mcu.id.clone();
                self.symbol_search.target_mcu.check_id(id.as_str());
            }

            ui.add(super::widgets::toggle(
                &mut self.symbol_search.target_mcu.id_rock,
            ));
            ui.label("LOCK");

            if self.symbol_search.target_mcu.id == "" {
                ui.label(RichText::new("Please input Target MCU name").color(Color32::RED));
            } else if self.symbol_search.target_mcu.id_not_found {
                ui.label(RichText::new("Target MCU not found").color(Color32::RED));
            }
        });
        ui.label("Memory usage");
        egui::Grid::new("memory_useage_grid")
            .num_columns(5)
            .spacing([5.0, 4.0])
            .striped(false)
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(" |     Region");
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(" |     Used Size");
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(" |     Region Size");
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(" |     %age Used |");
                });
                ui.end_row();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label("FLASH");
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(format!(
                        "{:>6.2} KB",
                        self.symbol_search.target_mcu.rom.used
                    ));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(format!("{:>3} KB", self.symbol_search.target_mcu.rom.size));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(format!(
                        "{:>6.2} %",
                        self.symbol_search.target_mcu.rom.percent
                    ));
                });
                ui.end_row();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label("RAM");
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(format!(
                        "{:>6.2} KB",
                        self.symbol_search.target_mcu.ram.used
                    ));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(format!("{:>3} KB", self.symbol_search.target_mcu.ram.size));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(format!(
                        "{:>6.2} %",
                        self.symbol_search.target_mcu.ram.percent
                    ));
                });
                ui.end_row();
            });

        ui.separator();
        ui.heading("Variable list");
        ui.horizontal(|ui| {
            ui.label("filler or variable name");
            ui.text_edit_singleline(&mut self.symbol_search.search_name);
            ui.label("  ");
            ui.checkbox(&mut self.symbol_search.expand_list_flag, "Expand List");
        });

        const CHECK_CLM: f32 = 15.;
        const ADDR_CLM: f32 = 85.;
        const TYPE_CLM: f32 = 120.;
        const SIZE_CLM: f32 = 40.;

        TableBuilder::new(ui)
            .striped(true)
            .auto_shrink([false; 2])
            .resizable(true)
            .vscroll(true)
            .drag_to_scroll(true)
            //.max_scroll_height(10.)
            .column(Column::initial(CHECK_CLM).resizable(false))
            .column(Column::initial(ADDR_CLM).resizable(true))
            .column(Column::initial(TYPE_CLM).resizable(true))
            .column(Column::initial(SIZE_CLM).resizable(true))
            .column(Column::remainder().resizable(true))
            .header(9.0, |mut header| {
                header.col(|_| {});
                header.col(|ui| {
                    ui.heading("Address");
                });
                header.col(|ui| {
                    ui.heading("Type");
                });
                header.col(|ui| {
                    ui.heading("Size");
                });
                header.col(|ui| {
                    ui.heading("Symbol Name");
                });
            })
            .body(|mut body| {
                for selected in self.symbol_search.selected_list.iter_mut() {
                    let re = Regex::new(r"\[[1-9]+").unwrap();
                    if !self.symbol_search.expand_list_flag && re.is_match(&selected.name) {
                        continue;
                    }
                    if selected
                        .name
                        .to_lowercase()
                        .contains(&self.symbol_search.search_name.to_lowercase())
                    {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut selected.is_selected, "")
                                    .on_hover_text("add watch list");
                            });
                            row.col(|ui| {
                                ui.label(format!("0x{:x}", selected.address));
                            });
                            row.col(|ui| {
                                ui.label(&selected.types);
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", &selected.size));
                            });
                            row.col(|ui| {
                                if ui
                                    .add(
                                        egui::Label::new(&selected.name)
                                            .sense(egui::Sense::click()),
                                    )
                                    .on_hover_text(&selected.name)
                                    .clicked()
                                {
                                    selected.is_selected = !selected.is_selected;
                                }
                            });
                        });
                    }
                }
            });
    }
    // ----------------------------------------------------------------------------

    fn check_probe(&mut self) {
        let probes = Probe::list_all();

        if probes.len() == 1 {
            self.probe_setting.select_sn =
                probes.get(0).and_then(|probe| probe.serial_number.clone());
        }
        self.probe_setting.probes = probes;
    }

    fn probe_setting_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Debug probe select");

        if ui.add(egui::Button::new("probe check")).clicked() {
            self.check_probe();
        }

        StripBuilder::new(ui)
            .size(Size::remainder().at_least(100.0)) // for the table
            .size(Size::exact(500.)) // for the source code link
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    egui::ScrollArea::horizontal().show(ui, |ui| {
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
                                for probe in self.probe_setting.probes.iter_mut() {
                                    body.row(18.0, |mut row| {
                                        row.col(|ui| {
                                            ui.radio_value(
                                                &mut self.probe_setting.select_sn,
                                                probe.serial_number.clone(),
                                                "",
                                            );
                                        });
                                        row.col(|ui| {
                                            ui.label(&probe.identifier);
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{:?}", probe.probe_type));
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{:?}", probe.vendor_id));
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{:?}", probe.product_id));
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{:?}", probe.serial_number));
                                        });
                                    });
                                }
                            });
                    });
                });
                strip.cell(|ui| {
                    ui.separator();
                    ui.heading("watch settings...");

                    ui.label(format!(
                        "select prove --> {:?}",
                        self.probe_setting.select_sn
                    ));
                    ui.label(format!(
                        "target mcu   --> {:?}",
                        self.symbol_search.target_mcu.id
                    ));

                    ui.separator();

                    ui.push_id(2, |ui| {
                        let watch_list = self.get_watch_list();
                        let mut to_remove = None;

                        TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .vscroll(true)
                            .drag_to_scroll(true)
                            .column(Column::exact(20.0))
                            .column(Column::initial(120.).resizable(true))
                            .column(Column::initial(160.).resizable(true))
                            .column(Column::remainder().resizable(true))
                            .header(9.0, |mut header| {
                                header.col(|_ui| {});
                                header.col(|ui| {
                                    ui.heading("Address");
                                });
                                header.col(|ui| {
                                    ui.heading("Type");
                                });
                                header.col(|ui| {
                                    ui.heading("Symbol Name");
                                });
                            })
                            .body(|mut body| {
                                for selected in watch_list {
                                    body.row(20.0, |mut row| {
                                        row.col(|ui| {
                                            if ui.button("x").clicked() {
                                                to_remove = Some(selected.name.clone());
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("0x{:x}", selected.address));
                                        });
                                        row.col(|ui| {
                                            ui.label(&selected.types);
                                        });
                                        row.col(|ui| {
                                            ui.label(&selected.name).on_hover_text(&selected.name);
                                        });
                                    });
                                }
                                if let Some(name) = to_remove {
                                    for item in &mut self.symbol_search.selected_list {
                                        if item.name == name {
                                            item.is_selected = false;
                                        }
                                    }
                                }
                            });
                    });
                });
            });
    }

    pub fn get_watch_list(&mut self) -> Vec<VariableInfo> {
        SelectableVariableInfo::pick_selected(&self.symbol_search.selected_list)
    }

    pub fn get_watch_setting(&mut self) -> WatchSetting {
        WatchSetting {
            target_mcu: self.symbol_search.target_mcu.id.clone(),
            probe_sn: self.probe_setting.select_sn.clone().unwrap_or_default(),
            watch_list: self.get_watch_list(),
        }
    }

    fn get_memory_usage(elf_file_path: &PathBuf) -> Option<(f64, f64)> {
        if let Ok((text, data, bss)) =
            ddbug_parser::File::parse(elf_file_path.to_str().unwrap().to_string()).and_then(|ctx| {
                let mut text_size = 0.0;
                let mut data_size = 0.0;
                let mut bss_size = 0.0;
                for sec in ctx.file().sections() {
                    let cast_size = sec.size() as f64;
                    match sec.name().unwrap() {
                        ".isr_vector" | ".text" | ".rodata" | ".ARM" | ".init_array"
                        | ".fini_array" => text_size += cast_size,
                        ".data" => data_size += cast_size,
                        ".bss" | "._user_heap_stack" => bss_size += cast_size,
                        _ => {}
                    }
                }
                Ok((text_size / 1024.0, data_size / 1024.0, bss_size / 1024.0))
            })
        {
            // ROM size, RAM size
            Some((text + data, data + bss))
        } else {
            None
        }
    }
}
