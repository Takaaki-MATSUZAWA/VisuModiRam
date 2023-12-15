#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::{path::PathBuf};
use eframe::egui::{self, Button, Color32};
use egui_extras::{TableBuilder, Column};
use shellexpand;

use egui_plot::{
    Arrows, AxisHints, Bar, BarChart, BoxElem, BoxPlot, BoxSpread, CoordinatesFormatter, Corner,
    GridInput, GridMark, HLine, Legend, Line, LineStyle, MarkerShape, Plot, PlotImage, PlotPoint,
    PlotPoints, PlotResponse, Points, Polygon, Text, VLine,
};

use probe_rs::{MemoryInterface, Permissions, Session, Probe};

mod gdb_parser;
use gdb_parser::{GdbParser, VariableList};

fn remap(value: f64, from_range: std::ops::RangeInclusive<f64>, to_range: std::ops::RangeInclusive<f64>) -> f64 {
    let (from_min, from_max) = (*from_range.start(), *from_range.end());
    let (to_min, to_max) = (*to_range.start(), *to_range.end());

    to_min + (value - from_min) * (to_max - to_min) / (from_max - from_min)
}

fn search_target_mcu_name(elf_file_path: &PathBuf) -> Option<String>{
    let project_name = elf_file_path.file_stem()?.to_str()?.to_string();
    let mut project_dir = elf_file_path.parent();

    while let Some(path) = project_dir {
        if path.file_name()?.to_str()? == project_name {
            break;
        }
        project_dir = path.parent();
    }

    let ioc_file_path = project_dir?.join(format!("{}.ioc", &project_name));
    if ioc_file_path.is_file() {
        let content = std::fs::read_to_string(&ioc_file_path).ok()?;
        for line in content.lines() {
            if line.starts_with("ProjectManager.DeviceId=") {
                return Some(line["ProjectManager.DeviceId=".len()..].to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    // > cargo test --all -- --nocapture test_search_target_mcu_name
    use super::*;

    const ELF_FILE: &str = r"~\Dropbox\project\STM32Cube\VScode_workspace\STSPIN32G4_mobility2022_motor\build\debug\build\STSPIN32G4_mobility2022_motor.elf";
    
    #[test]
    fn test_search_target_mcu_name() {
        let path = PathBuf::from(format!("{}",  shellexpand::tilde(&ELF_FILE.to_string())));
        let result = search_target_mcu_name(&path);
        assert!(result.is_some());
        let unwrapped_result = result.unwrap();
        println!("{}", unwrapped_result);
        assert_eq!(unwrapped_result, "STM32G431VBTx");
    }

    #[test]
    fn test_get_symbol_size(){
        let elf_path = format!("{}",  shellexpand::tilde(&ELF_FILE.to_string()));

        let mut gdb_parser = GdbParser::launch(&PathBuf::from(&elf_path)).unwrap();

        let size = gdb_parser.get_variable_size(&"ADCs.Channel".to_string());
        println!("{:?}", size);

        
        gdb_parser.scan_variables_none_blocking_start();
        println!("scan start");

        loop{
            let now_progress = gdb_parser.get_scan_progress();
            print!("\rprocess {:3} %", (now_progress * 100.0) as i32);
            std::io::stdout().flush();
            std::thread::sleep(std::time::Duration::from_millis(100));

            if now_progress == 1.0{
                break;
            }
        }

        println!("\nscan complete");

        let variable_list = gdb_parser.load_variable_list();

        for num in 0..10{
            //let size = 0;
            println!("add:{}, type:{}, size:{:?}, name:{}", variable_list[num].address, variable_list[num].types, variable_list[num].size, variable_list[num].name);

        }
         
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(960.0, 480.0)),
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
    input_elf_path: String,
    elf_path: String,
    search_name: String,
    variable_list: Vec<VariableList>,
    search_results_list: Vec<VariableList>,
    selected_list: Vec<MainVariableList>,
    gdb_parser: Option<GdbParser>,
    elf_file_is_not_found: bool,
    target_mcu_id: String,
    probes: Vec<probe_rs::DebugProbeInfo>,
    time: f64,

}

struct MainVariableList {
    name :String,
    types:String,
    address:String,
    is_selected: bool,
}


impl Default for STM32EguiMonitor {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            input_elf_path: Default::default(),
            elf_path: Default::default(),
            search_name: Default::default(),
            variable_list: Vec::new(),
            search_results_list: Vec::new(),
            selected_list: Vec::new(),
            gdb_parser: None,
            elf_file_is_not_found: false,
            target_mcu_id: Default::default(),
            probes: Vec::new(),
            time: 0.0,
        }
    }
}

impl eframe::App for STM32EguiMonitor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();


        egui::SidePanel::left("setting").show(ctx, |ui| {
            ui.heading("ELF File Viewer");

            if ui.add(egui::Button::new("prove check")).clicked() {
                self.probes = Vec::new();
                self.probes = Probe::list_all();
            }
            ui.push_id(0, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .max_scroll_height(40.)
                    .vscroll(true)
                    .drag_to_scroll(true)
                    .column(Column::auto().resizable(true))
                    .column(Column::auto().resizable(true))
                    .column(Column::auto().resizable(true))
                    .column(Column::auto().resizable(true))
                    .column(Column::auto().resizable(true))
                    .header(20.0, |mut header| {
                        
                        header.col(|ui|{
                            ui.heading("name");
                        });
                        
                        header.col(|ui| {
                            ui.heading("type");
                        });
                        header.col(|ui| {
                            ui.heading("vnd_id");
                        });
                        header.col(|ui| {
                            ui.heading("prd_id");
                        });
                        header.col(|ui| {
                            ui.heading("SN");
                        });
                    })
                    .body(|mut body| {
                        for probe in &self.probes {
                            body.row(18.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(&probe.identifier);
                                });
                                row.col(|ui| {
                                    ui.label(format!("{:?}",probe.probe_type));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{:?}",probe.vendor_id));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{:?}",probe.product_id));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{:?}",probe.serial_number));
                                });
                            });
                        }
                    });
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("target MUC:");
                ui.text_edit_singleline(&mut self.target_mcu_id);
            });

            ui.horizontal(|ui| {
                ui.label("Path to ELF file:");
                ui.text_edit_singleline(&mut self.input_elf_path);
                self.elf_path = format!("{}",  shellexpand::tilde(&self.input_elf_path));

                if ui.button("Load").clicked() {
                    if !std::path::Path::new(&self.elf_path).exists() {
                        self.elf_file_is_not_found = true;
                    }else{
                        self.elf_file_is_not_found = false;
                        
                        if let Some(mcu_id) = search_target_mcu_name(&PathBuf::from(&self.elf_path)){
                            self.target_mcu_id = mcu_id;
                        }

                        if let Ok(gdb_parser) = GdbParser::launch(&PathBuf::from(&self.elf_path)) {
                            self.variable_list = Vec::new();
    
                            self.gdb_parser = Some(gdb_parser);
                            if let Some(gdb_parser) = &mut self.gdb_parser {
                                gdb_parser.scan_variables_none_blocking_start();
                                println!("scan start");
                            }
                        }else{
                            println!("failed file load");
                        }
                    }
                }
                if self.elf_file_is_not_found{
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
                        for vals in self.variable_list.clone(){
                            self.selected_list.push(MainVariableList { name: vals.name, types: vals.types, address: vals.address, is_selected: false });
                        }
                    }
                }
            }
            
            ui.add(egui::ProgressBar::new(now_progress)
                .text(prgres_text)
                .animate(prgress_anime)
            );
            
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Search variable name");
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
                        header.col(|ui|{
                            ui.set_width(10.)
                        });
                        
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
                                    if self.selected_list[i].name.to_lowercase().contains(&self.search_name.to_lowercase()){
                                        row.col(|ui|{
                                            ui.checkbox(&mut self.selected_list[i].is_selected, "");
                                        });
                                        row.col(|ui| {
                                            ui.label(&self.selected_list[i].address);
                                        });
                                        row.col(|ui| {
                                            ui.label(&self.selected_list[i].types);
                                        });
                                        row.col(|ui| {
                                            ui.label(&self.selected_list[i].name).on_hover_text(&self.selected_list[i].name);
                                        });
                                    }
                            });
                        }
                    });
                });
        });

        egui::CentralPanel::default().show(ctx, |cui| {
            cui.heading("Centor Panel");
            egui::Window::new("Window").show(ctx, |cui| {
                cui.heading("STM32EguiMonitor");
                cui.horizontal(|cui| {
                    let name_label = cui.label("Your name: ");
                    cui.text_edit_singleline(&mut self.name)
                        .labelled_by(name_label.id);
                });
                cui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
                if cui.button("Click each year").clicked() {
                    self.age += 1;
                }
    
                cui.label(format!("Hello '{}', age {}", self.name, self.age));
            });

                egui::Window::new("Graph Window").show(ctx, |ui| {
                    use std::f32::consts::{PI, TAU};
                    self.time = ctx.input(|input_state| input_state.time);
                    
                    let mut plot = Plot::new("plot_demo")
                        .legend(Legend::default())
                        .y_axis_width(4);

                        plot.show(ui, |plot_ui| {
                            plot_ui.line({
                                let n = 512;
                                let circle_points: PlotPoints = (0..=n)
                                    .map(|i| {
                                        let t = remap(i as f64, 0.0..=(n as f64), 0.0..=TAU.into());
                                        //let r = self.circle_radius;
                                        let r = 2.0;
                                        [
                                            //r * t.cos() + self.circle_center.x as f64,
                                            //r * t.sin() + self.circle_center.y as f64,
                                            r * t.cos() + 1.0 as f64,
                                            r * t.sin() + 2.0 as f64,
                                        ]
                                    })
                                    .collect();
                                Line::new(circle_points)
                                    .color(Color32::from_rgb(100, 200, 100))
                                    .style(LineStyle::Solid)
                                    .name("circle")
                            });
                            plot_ui.line({
                                let time = self.time;
                                Line::new(PlotPoints::from_explicit_callback(
                                    move |x| 0.5 * (2.0 * x).sin() * time.sin(),
                                    ..,
                                    512,
                                ))
                                .color(Color32::from_rgb(200, 100, 100))
                                .style(LineStyle::Solid)
                                .name("wave")
                            });
                        })
                        .response
                });
            // watch valiables list
            egui::Window::new("watch valiables list").show(ctx, |ui| {
                cui.heading("watch valiables list");
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
                        for vals in &self.selected_list {
                            body.row(18.0, |mut row| {
                                if vals.is_selected{
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
