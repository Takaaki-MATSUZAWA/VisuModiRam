use eframe::egui::{self, Color32};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

use super::{
    widgets::{self, WidgetApp},
    WidgetWindow,
};
use crate::debugging_tools::ProbeInterface;

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct MainMonitorTab {
    widgets: Vec<Box<WidgetWindow>>,
    window_cnt: u32,

    pub probe_if: ProbeInterface,
}

impl eframe::App for MainMonitorTab {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        //let window_width = ctx.available_rect().width();

        egui::SidePanel::left("control")
            .resizable(true)
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.heading("watch control");
                StripBuilder::new(ui)
                    .size(Size::remainder().at_least(300.0))
                    .size(Size::exact(1000.))
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            if ui.button("switch to monitor").clicked() {
                                for wid in &mut self.widgets {
                                    wid.switch_tab_to(super::widgets::Anchor::MonitorTab);
                                }
                            }

                            ui.separator();
                            if ui.button("watch start").clicked() {
                                for wid in &mut self.widgets {
                                    wid.set_probe_to_app(self.probe_if.clone());
                                    wid.switch_tab_to(super::widgets::Anchor::MonitorTab);
                                }
                                self.probe_if
                                    .watching_start(std::time::Duration::from_millis(1));
                            }

                            if ui.button("stop").clicked() {
                                self.probe_if.watching_stop();
                            }
                        });
                        strip.cell(|ui| {
                            ui.separator();
                            self.watch_setting_ui(ui);
                        });
                        //ui.separator();
                    });
            });

        egui::SidePanel::right("widgets")
            .resizable(false)
            .default_width(140.0)
            .show(ctx, |ui| {
                ui.heading("monitor app list");
                ui.separator();

                // ----------------------------------------------------------------------------
                self.add_widget_botton(
                    ui,
                    "add window",
                    "window",
                    Box::new(widgets::WidgetTest::default()),
                );
                // ----------------------------------------------------------------------------
                self.add_widget_botton(
                    ui,
                    "add graph",
                    "graph",
                    Box::new(widgets::GraphMonitor::default()),
                );
                // ----------------------------------------------------------------------------
                self.add_widget_botton(
                    ui,
                    "add table view",
                    "table_view",
                    Box::new(widgets::TableView::default()),
                );
                // ----------------------------------------------------------------------------
                self.add_widget_botton(
                    ui,
                    "add edit table",
                    "edit_table",
                    Box::new(widgets::EditTable::default()),
                );
                // ----------------------------------------------------------------------------
                self.add_widget_botton(
                    ui,
                    "add slider UI",
                    "Sliders",
                    Box::new(widgets::Sliders::default()),
                );
                // ----------------------------------------------------------------------------
                self.add_widget_botton(
                    ui,
                    "add toggle switchs",
                    "toggle switchs",
                    Box::new(widgets::ToggleSwitch::default()),
                );
                // ----------------------------------------------------------------------------

                for wid in &mut self.widgets {
                    wid.fetch_watch_list(&self.probe_if.setting.watch_list);
                }

                ui.separator();
                if ui.button("Organize windows").clicked() {
                    ui.ctx().memory_mut(|mem| mem.reset_areas());
                }
                ui.separator();
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(false)
                    .vscroll(true)
                    .column(Column::initial(115.).resizable(false))
                    .column(Column::initial(25.).resizable(false))
                    .header(9.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("window name");
                            //ui.set_width(100.0);
                        });
                        header.col(|_ui| {});
                    })
                    .body(|mut body| {
                        let mut to_remove = None;
                        let widget_names: Vec<_> =
                            self.widgets.iter().map(|w| w.name.clone()).collect();

                        for wid in &mut self.widgets {
                            body.row(15.0, |mut row| {
                                row.col(|ui| {
                                    let mut text = wid.name.clone();
                                    let res = ui.text_edit_singleline(&mut text);

                                    if res.changed() {
                                        if !widget_names.contains(&text) {
                                            wid.name = text;
                                        }
                                    }
                                    if res.hovered() {
                                        ui.ctx().debug_painter().debug_rect(
                                            wid.rect,
                                            Color32::RED,
                                            "",
                                        );
                                    }
                                });
                                row.col(|ui| {
                                    let res = ui.button("X");
                                    if res.clicked() {
                                        to_remove = Some(wid.id.clone());
                                    }
                                    if res.hovered() {
                                        ui.ctx().debug_painter().debug_rect(
                                            wid.rect,
                                            Color32::RED,
                                            "",
                                        );
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
            let mut to_remove = None;

            for app in &mut self.widgets {
                let mut open = true;
                app.update(ctx, frame, &mut open);

                if open == false {
                    to_remove = Some(app.id.clone());
                }
            }
            if let Some(index) = to_remove {
                self.widgets.retain(|x| x.id != index);
            }
        });
    }
}

impl MainMonitorTab {
    fn watch_setting_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("watch settings...");

        ui.label(format!(
            "probe --> {:?}",
            self.probe_if.setting.probe_sn != ""
        ));
        ui.label(format!("mcu   --> {:?}", self.probe_if.setting.target_mcu));

        ui.separator();
        ui.heading("watch list");
        egui::Grid::new("watch_list")
            .num_columns(1)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                let watch_list = self.probe_if.setting.watch_list.clone();
                for val in watch_list {
                    ui.label(&val.name);
                    ui.end_row();
                }
            });
    }

    fn add_widget_botton(
        &mut self,
        ui: &mut egui::Ui,
        text: &str,
        title: &str,
        widget: Box<dyn WidgetApp>,
    ) {
        if ui.button(text).clicked() {
            self.window_cnt += 1;
            let widget_window = WidgetWindow::new(
                self.window_cnt,
                format!("{}_{}", self.window_cnt, title),
                widget,
            );

            self.widgets.push(Box::new(widget_window));
        }
    }
}
