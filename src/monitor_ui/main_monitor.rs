use eframe::egui::{self, Button, Color32, Label};
use egui_extras::{Column, TableBuilder, StripBuilder, Size};

use super::WidgetWindow;

#[derive(Default)]
pub struct MainMonitorTab {
    widgets: Vec<Box<WidgetWindow>>,
    window_cnt: u32,

    pub probe_if: ProbeInterface2,
}

use crate::{monitor_ui::widgetTest, debugging_tools::ProbeInterface2};

impl eframe::App for MainMonitorTab {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        //let window_width = ctx.available_rect().width();

        egui::SidePanel::left("control")
            .resizable(true)
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.heading("watch control");
                StripBuilder::new(ui)
                    .size(Size::remainder().at_least(300.0)) // for the table
                    .size(Size::exact(1000.)) 
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            ui.label("test test");

                            if ui.button("watch start").clicked(){
                                self.probe_if.watching_start(std::time::Duration::from_millis(100));
                            }

                            if ui.button("stop").clicked() {
                                self.probe_if.watching_stop();
                            }
                        });
                        strip.cell(|ui|{
                            ui.separator();
                            self.watch_setting_ui(ui);
                        });
                //ui.separator();
                    });
            });

        egui::SidePanel::right("widgets")
            .resizable(true)
            .default_width(140.0)
            .show(ctx, |ui| {
                ui.heading("monitor app list");
                ui.separator();

                let mut add_flag = false;

                if ui.button("add window").clicked(){
                    self.window_cnt += 1;
                    let widget_window = WidgetWindow::new(
                        self.window_cnt,
                        format!("window {}",self.window_cnt),
                        Box::new(widgetTest::new(
                            "bbb bbb ".to_string(),
                            self.window_cnt*10))
                    );
                    self.widgets.push(Box::new(widget_window));
                    add_flag = true;
                }
                if add_flag{
                    for wid in &mut self.widgets{
                        wid.fetch_watch_list(&self.probe_if.setting.watch_list);
                    }
                }

                ui.separator();
                TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .vscroll(true)
                .column(Column::initial(120.).resizable(true))
                .column(Column::initial(20.).resizable(true))
                .header(9.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("window name");
                        //ui.set_width(100.0);
                    });
                    header.col(|_ui| {});
                })
                .body(|mut body| {
                    let mut to_remove = None;

                    for wid in &mut self.widgets{
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                let mut text = wid.name.clone();
                                let res = ui.text_edit_singleline(&mut text);

                                if res.changed(){
                                    wid.name = text;
                                }
                                if res.hovered(){
                                    ui.ctx().debug_painter().debug_rect(wid.rect, Color32::RED, "");
                                }
                            });
                            row.col(|ui| {
                                let res = ui.button("x");
                                if res.clicked() {
                                    to_remove = Some(wid.id.clone());
                                }
                                if res.hovered(){
                                    ui.ctx().debug_painter().debug_rect(wid.rect, Color32::RED, "");
                                }
                            });
                        });
                    }

                    if let Some(index) = to_remove {
                        self.widgets.retain(|x| x.id != index);
                    }
                }); 
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("main panel");

            for app in &mut self.widgets{
                app.update(ctx, frame);
            }
        });
    }
}

impl  MainMonitorTab {
    fn watch_setting_ui(&mut self, ui: &mut egui::Ui){
        ui.heading("watch settings...");

        ui.label(format!(
            "probe --> {:?}",
            self.probe_if.setting.probe_sn != ""
        ));
        ui.label(format!(
            "mcu   --> {:?}",
            self.probe_if.setting.target_mcu
        ));

        ui.separator();
        ui.heading("watch list");
        egui::Grid::new("watch_list")
            .num_columns(1)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                let watch_list = self.probe_if.setting.watch_list.clone();
                for val in watch_list{
                    ui.label(&val.name);
                    ui.end_row();
                }
            });

        #[cfg(predicate)]
        ui.push_id(2, |ui| {
            let watch_list = self.probe_if.setting.watch_list.clone();
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
                });
            });
    }
}
