use eframe::egui::{self, Button, Color32};
use egui_extras::{Column, TableBuilder};

use super::WidgetWindow;
use super::Widget;
use super::WidgetApp;

#[derive(Default)]
pub struct MainMonitorTab {
    widgets: Vec<Box<WidgetWindow>>,
    window_cnt: u32,
}

#[cfg(disable)]
impl MainMonitorTab {
    fn setup(&mut self) -> &mut Self {
        self.widgets.push(Box::new(Widget::new(
            0,
            "test 0".to_string(),
            Box::new(widgetTest::new("aaa".to_string(), 42)),
        )));
        self.widgets.push(Widget::new(
            1,
            "test 1".to_string(),
            Box::new(widgetTest::new("bbb bbb ".to_string(), 12)),
        ));

        self
    }
}

use crate::monitor_ui::widgetTest;

impl eframe::App for MainMonitorTab {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        //let window_width = ctx.available_rect().width();

        egui::SidePanel::left("control")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("probe control panel");
                ui.separator();
                ui.label("text 1");
            });

        egui::SidePanel::right("widgets")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("monitor app list");
                ui.separator();
                ui.label("text 1");

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
                }

                ui.separator();
                TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .vscroll(true)
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .header(9.0, |mut header| {
                    header.col(|ui| {
                        ui.label("window name");
                        ui.set_width(50.0);
                    });
                    header.col(|ui| {
                        //ui.heading("Symbol");
                    });
                })
                .body(|mut body| {
                    let mut to_remove = None;

                    for wid in &mut self.widgets{
                        body.row(9.0, |mut row| {
                            row.col(|ui| {
                                ui.label(&wid.name);
                            });
                            row.col(|ui| {
                                if ui.button("x").clicked() {
                                    to_remove = Some(wid.id.clone());
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
