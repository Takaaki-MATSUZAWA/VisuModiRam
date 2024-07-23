use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

use rfd::FileDialog;
use std::path::PathBuf;

use crate::debugging_tools::*;
use probe_rs::Probe;

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct SymbolSearch {
    input_elf_path: String,
    search_name: String,
    variable_list: Vec<VariableInfo>,
    selected_list: Vec<SelectableVariableInfo>,
    #[cfg_attr(feature = "serde", serde(skip))]
    gdb_parser: Option<GdbParser>,
    project_name: String,
    target_mcu_id: String,

    rom_size: f64,
    ram_size: f64,
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
                        self.symbol_search.target_mcu_id = mcu_id;
                    } else {
                        self.symbol_search.target_mcu_id = "".to_string();
                    }

                    if let Ok(gdb_parser) = GdbParser::launch(&PathBuf::from(&elf_path)) {
                        self.symbol_search.variable_list = Vec::new();

                        self.symbol_search.gdb_parser = Some(gdb_parser);
                        if let Some(gdb_parser) = &mut self.symbol_search.gdb_parser {
                            gdb_parser.scan_variables_none_blocking_start();
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

        if let Some(gdb_parser) = &mut self.symbol_search.gdb_parser {
            let now_progress = gdb_parser.get_scan_progress();

            if now_progress < 1.0 {
                self.progress_bar
                    .set_progress(ProgresState::SymbolSearching, now_progress);
            } else {
                if self.symbol_search.variable_list.is_empty() {
                    self.symbol_search.variable_list = gdb_parser.load_variable_list();
                    if !self.symbol_search.variable_list.is_empty() {
                        SelectableVariableInfo::fetch(
                            &self.symbol_search.variable_list,
                            &mut self.symbol_search.selected_list,
                        );
                        self.progress_bar
                            .complete(ProgresState::SymbolSearchComplite);

                        let elf_path =
                            format!("{}", shellexpand::tilde(&self.symbol_search.input_elf_path));
                        if let Ok(res) = Self::get_memory_usage(&PathBuf::from(&elf_path)) {
                            self.symbol_search.rom_size = res.0;
                            self.symbol_search.ram_size = res.1;
                        } else {
                            self.symbol_search.rom_size = 0.0;
                            self.symbol_search.ram_size = 0.0;
                        }
                    }
                }
            }
        }

        if self.probe_setting.flash_probe_if.get_flash_progress().state != FlashProgressState::None
        {
            let flash_progress = self.probe_setting.flash_probe_if.get_flash_progress();

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
        }

        self.progress_bar.show(ui);

        ui.separator();
        ui.heading("Infomation");
        ui.label(format!(
            "Project Name : {}",
            self.symbol_search.project_name
        ));
        ui.horizontal(|ui| {
            ui.label("Target MCU : ");
            ui.text_edit_singleline(&mut self.symbol_search.target_mcu_id);

            if self.symbol_search.target_mcu_id == "" {
                ui.label(RichText::new("Please input Target MCU name").color(Color32::RED));
            }
        });
        ui.label("Memory usage");
        ui.label(format!("  ROM : {:.2} KByte", self.symbol_search.rom_size));
        ui.label(format!("  RAM : {:.2} KByte", self.symbol_search.ram_size));

        ui.separator();
        ui.heading("Variable list");
        ui.horizontal(|ui| {
            ui.label("filler or variable name");
            ui.text_edit_singleline(&mut self.symbol_search.search_name);
        });

        let window_width = ctx.available_rect().width() / 2.0;

        const CHECK_CLM: f32 = 15.;
        const ADDR_CLM: f32 = 85.;
        const TYPE_CLM: f32 = 120.;
        const SIZE_CLM: f32 = 40.;

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .vscroll(true)
            .drag_to_scroll(true)
            //.max_scroll_height(10.)
            .column(Column::initial(CHECK_CLM).resizable(false))
            .column(Column::initial(ADDR_CLM).resizable(true))
            .column(Column::initial(TYPE_CLM).resizable(true))
            .column(Column::initial(SIZE_CLM).resizable(true))
            .column(
                Column::initial(window_width - (CHECK_CLM + ADDR_CLM + TYPE_CLM + SIZE_CLM + 50.0))
                    .at_least(50.0)
                    .resizable(true),
            )
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
                        self.symbol_search.target_mcu_id
                    ));

                    ui.push_id(2, |ui| {
                        let watch_list = self.get_watch_list();
                        let mut to_remove = None;

                        TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .vscroll(true)
                            .drag_to_scroll(true)
                            .column(Column::initial(120.).resizable(true))
                            .column(Column::initial(160.).resizable(true))
                            .column(Column::initial(290.).at_least(50.0).resizable(true))
                            .column(Column::auto().at_least(30.0).resizable(true))
                            .header(9.0, |mut header| {
                                header.col(|ui| {
                                    ui.heading("Address");
                                });
                                header.col(|ui| {
                                    ui.heading("Type");
                                });
                                header.col(|ui| {
                                    ui.heading("Symbol Name");
                                });
                                header.col(|_ui| {});
                            })
                            .body(|mut body| {
                                for selected in watch_list {
                                    body.row(20.0, |mut row| {
                                        row.col(|ui| {
                                            ui.label(format!("0x{:x}", selected.address));
                                        });
                                        row.col(|ui| {
                                            ui.label(&selected.types);
                                        });
                                        row.col(|ui| {
                                            ui.label(&selected.name).on_hover_text(&selected.name);
                                        });
                                        row.col(|ui| {
                                            if ui.button("x").clicked() {
                                                to_remove = Some(selected.name.clone());
                                            }
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
            target_mcu: self.symbol_search.target_mcu_id.clone(),
            probe_sn: self.probe_setting.select_sn.clone().unwrap_or_default(),
            watch_list: self.get_watch_list(),
        }
    }

    fn get_memory_usage(elf_file_path: &PathBuf) -> Result<(f64, f64)> {
        let name = ::std::env::var("SIZE_ARM_BINARY").unwrap_or("arm-none-eabi-size".to_string());
        let output = std::process::Command::new(name)
            .arg(elf_file_path)
            .output()?;

        if !output.status.success() {
            //return Err(gdb_parser::Error::ParseError::new("arm-none-eabi-size コマンドの実行に失敗しました".to_string()));
            return Err(Error::ParseError);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.trim().split('\n').collect();
        if lines.len() < 2 {
            //return Err("予期せぬ出力形式です".to_string());
            return Err(Error::ParseError);
        }

        let values: Vec<&str> = lines[1].split_whitespace().collect();
        if values.len() < 4 {
            //return Err("予期せぬ出力形式です".to_string());
            return Err(Error::ParseError);
        }

        let text = values[0].parse::<f64>().unwrap() / 1024.0;
        let data = values[1].parse::<f64>().unwrap() / 1024.0;
        let bss = values[2].parse::<f64>().unwrap() / 1024.0;

        // ROM size, RAM size
        Ok((text + data, data + bss))
    }
}
