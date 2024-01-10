use eframe::egui::{self, Color32};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use std::time::Duration;

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
    #[cfg_attr(feature = "serde", serde(skip))]
    remove_que: Option<u32>,
    watch_duration_ms: u64,

    hide_title_bar: bool,
    move_and_resize_lock: bool,

    pub probe_if: ProbeInterface,
}

impl eframe::App for MainMonitorTab {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
                            ui.horizontal(|ui| {
                                let now_watching = &self.probe_if.now_watching();
                                ui.add_enabled_ui(*now_watching == false, |ui| {
                                    ui.label("Duration :");
                                    ui.add(
                                        egui::DragValue::new(&mut self.watch_duration_ms)
                                            .suffix("[ms]")
                                            .clamp_range(1..=1000)
                                            .speed(10),
                                    );
                                });
                            });
                            if ui.button("watch start").clicked() {
                                for wid in &mut self.widgets {
                                    wid.set_probe_to_app(self.probe_if.clone());
                                    wid.switch_tab_to(super::widgets::Anchor::MonitorTab);
                                }
                                self.probe_if
                                    .watching_start(Duration::from_millis(self.watch_duration_ms));
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
            .default_width(110.)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Add Monitor");
                ui.separator();
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    ui.strong("Viewer");
                    // ----------------------------------------------------------------------------
                    #[cfg(disable)]
                    self.add_widget_botton(
                        ui,
                        "window",
                        "window",
                        Box::new(widgets::WidgetTest::default()),
                    );
                    // ----------------------------------------------------------------------------
                    self.add_widget_botton(
                        ui,
                        "Plot",
                        "Plot",
                        Box::new(widgets::GraphMonitor::default()),
                    );
                    // ----------------------------------------------------------------------------
                    self.add_widget_botton(
                        ui,
                        "Table view",
                        "table_view",
                        Box::new(widgets::TableView::default()),
                    );
                    // ----------------------------------------------------------------------------
                    self.add_widget_botton(
                        ui,
                        "Gauge view",
                        "gauge_view",
                        Box::new(widgets::Gauges::default()),
                    );
                    // ----------------------------------------------------------------------------

                    ui.separator();
                    ui.strong("Editor");
                    self.add_widget_botton(
                        ui,
                        "Edit Table",
                        "edit_table",
                        Box::new(widgets::EditTable::default()),
                    );
                    // ----------------------------------------------------------------------------
                    self.add_widget_botton(
                        ui,
                        "Slider UI",
                        "Sliders",
                        Box::new(widgets::Sliders::default()),
                    );
                    // ----------------------------------------------------------------------------
                    self.add_widget_botton(
                        ui,
                        "Toggle Switchs",
                        "toggle_switchs",
                        Box::new(widgets::ToggleSwitch::default()),
                    );
                    // ----------------------------------------------------------------------------
                    self.add_widget_botton(
                        ui,
                        "Push Buttons",
                        "push_buttons",
                        Box::new(widgets::PushButton::default()),
                    );
                    // ----------------------------------------------------------------------------

                    for wid in &mut self.widgets {
                        wid.fetch_watch_list(&self.probe_if.setting.watch_list);
                    }

                    ui.separator();
                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory_mut(|mem| mem.reset_areas());
                    }

                    egui::Grid::new("window_setting_switchs")
                        .num_columns(2)
                        .spacing([5.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Show title bar");
                            let mut enable = !self.hide_title_bar;
                            if ui.add(super::widgets::toggle(&mut enable)).changed() {
                                for wid in &mut self.widgets {
                                    wid.show_title_bar(enable);
                                }
                            }
                            self.hide_title_bar = !enable;
                            ui.end_row();

                            ui.label("window lock");
                            if ui
                                .add(super::widgets::toggle(&mut self.move_and_resize_lock))
                                .changed()
                            {
                                for wid in &mut self.widgets {
                                    wid.lock(self.move_and_resize_lock);
                                }
                            }
                            ui.end_row();
                        });
                });
                ui.separator();
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(false)
                    .vscroll(true)
                    .column(Column::initial(90.).resizable(false))
                    .column(Column::auto_with_initial_suggestion(25.).resizable(false))
                    .header(22.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("window name");
                        });
                        header.col(|_ui| {});
                    })
                    .body(|mut body| {
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

                                    if res.clicked() {
                                        ui.ctx().move_to_top(wid.layer_id);
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
                                        Self::remove_widget_que(&mut self.remove_que, &wid.id);
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
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("main panel");
        });

        for app in &mut self.widgets {
            let mut open = true;
            app.update(ctx, frame, &mut open);

            if open == false {
                Self::remove_widget_que(&mut self.remove_que, &app.id);
            }
        }

        self.remove_widget_exec();
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
                !self.hide_title_bar,
                self.move_and_resize_lock,
                widget,
            );

            self.widgets.push(Box::new(widget_window));
        }
    }

    fn remove_widget_que(que: &mut Option<u32>, id: &u32) {
        *que = Some(id.clone());
    }

    fn remove_widget_exec(&mut self) {
        if let Some(id) = self.remove_que {
            self.widgets.retain(|x| x.id != id);
        }
    }
}
