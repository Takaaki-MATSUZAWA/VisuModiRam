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
struct SymbolSearch {
    input_elf_path: String,
    search_name: String,
    variable_list: Vec<VariableInfo>,
    selected_list: Vec<SelectableVariableInfo>,
    gdb_parser: Option<GdbParser>,
    target_mcu_id: String,
}

#[derive(Default)]
struct ProbeSetting {
    probes: Vec<probe_rs::DebugProbeInfo>,
    select_sn: Option<String>,
}

// ----------------------------------------------------------------------------

#[derive(Default)]
pub struct SettingTab2 {
    symbol_search: SymbolSearch,
    probe_setting: ProbeSetting,
}

impl eframe::App for SettingTab2 {
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

impl SettingTab2 {
    fn symbol_search_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("ELF file loader");
        ui.horizontal(|ui| {
            ui.label("Path to ELF file :");
            ui.text_edit_singleline(&mut self.symbol_search.input_elf_path);

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("browse…").clicked() {
                if let Some(path) = FileDialog::new().pick_file() {
                    self.symbol_search.input_elf_path = path
                        .to_str()
                        .ok_or_else(|| "Failed to convert path to string")
                        .unwrap()
                        .to_string();
                }
            }

            let elf_path = format!("{}", shellexpand::tilde(&self.symbol_search.input_elf_path));
            let is_elf_file_exixt = std::path::Path::new(&elf_path).exists();

            if ui.button("Load").clicked() {
                self.check_probe();
                if is_elf_file_exixt {
                    if let Some(mcu_id) =
                        crate::debugging_tools::search_target_mcu_name(&PathBuf::from(&elf_path))
                    {
                        self.symbol_search.target_mcu_id = mcu_id;
                    }

                    if let Ok(gdb_parser) = GdbParser::launch(&PathBuf::from(&elf_path)) {
                        self.symbol_search.variable_list = Vec::new();

                        self.symbol_search.gdb_parser = Some(gdb_parser);
                        if let Some(gdb_parser) = &mut self.symbol_search.gdb_parser {
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

        if let Some(gdb_parser) = &mut self.symbol_search.gdb_parser {
            now_progress = gdb_parser.get_scan_progress();
            prgress_anime = true;

            if now_progress < 1.0 {
                prgres_text = "Loading...";
            } else {
                prgres_text = "complete";
                prgress_anime = false;

                if self.symbol_search.variable_list.is_empty() {
                    self.symbol_search.variable_list = gdb_parser.load_variable_list();

                    self.symbol_search.selected_list =
                        SelectableVariableInfo::generate(&self.symbol_search.variable_list);
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
            ui.label("Target MCU : ");
            ui.text_edit_singleline(&mut self.symbol_search.target_mcu_id);
        });

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
                                ui.label(&selected.address);
                            });
                            row.col(|ui| {
                                ui.label(&selected.types);
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", &selected.size));
                            });
                            row.col(|ui| {
                                ui.label(&selected.name).on_hover_text(&selected.name);
                            });
                        });
                    }
                }
            });
    }

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
                                            ui.label(&selected.address);
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
}
