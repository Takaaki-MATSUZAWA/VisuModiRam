mod graph_test;
mod probe_if_test;

use std::vec;

pub use graph_test::GraphTest;
pub use probe_if_test::ProbeIfTest;

mod widget_test;
pub use widget_test::widgetTest;
// ----------------------------------------------------------------------------
use eframe::{
    egui::{self, Color32, Pos2, Rect, Vec2},
    App,
};
use egui_extras::{Column, TableBuilder};
use std::sync::Arc;

use crate::debugging_tools::VariableInfo;

use super::symbol_search::SelectableVariableInfo;

pub struct Widget<'a> {
    pub id: u32,
    pub name: String,
    watch_list: Option<Arc<Vec<VariableInfo>>>,
    pub wiget_ui: Box<dyn WidgetApp<'a>>,
}

impl<'a> Widget<'a> {
    pub fn new(id: u32, name: String, wiget_ui: Box<dyn WidgetApp<'a>>) -> Self {
        Self {
            id,
            name,
            watch_list: None,
            wiget_ui,
        }
    }

    pub fn set_watch_list_ptr(&mut self, watch_list_ptr: &Vec<VariableInfo>) {
        self.watch_list = Some(Arc::new(watch_list_ptr.clone()));
    }

    pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let mut _in_watch_list = Vec::new();

        if let Some(watch_list) = &mut self.watch_list {
            self.wiget_ui.fetch_watch_list(watch_list);

            for val in watch_list.iter() {
                _in_watch_list.push(val);
            }
        }

        egui::Window::new(&self.name).show(ctx, |ui| {
            self.wiget_ui.ui(ui);

            ui.separator();
            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .vscroll(true)
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Address");
                        ui.set_width(80.0);
                    });
                    header.col(|ui| {
                        ui.heading("Symbol");
                    });
                })
                .body(|mut body| {
                    for vals in &_in_watch_list {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.label(&vals.address);
                            });
                            row.col(|ui| {
                                ui.label(&vals.name);
                            });
                        });
                    }
                });
        });
    }
}

//pub trait WidgetApp<'a>: eframe::App {
pub trait WidgetApp<'a> {
    fn ui(&mut self, ui: &mut egui::Ui);
    fn fetch_watch_list(&mut self, watch_list: &Vec<crate::debugging_tools::VariableInfo>);
}

#[cfg(disable)]
impl<'a> eframe::App for dyn WidgetApp<'a> {
    fn update(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }
}

// ----------------------------------------------------------------------------

pub struct Widget2 {
    pub id: u32,
    pub name: String,
    watch_list: Option<Arc<Vec<VariableInfo>>>,
    pub wiget_ui: Box<dyn WidgetApp2>,
}

impl Widget2 {
    pub fn new(id: u32, name: String, wiget_ui: Box<dyn WidgetApp2>) -> Self {
        Self {
            id,
            name,
            watch_list: None,
            wiget_ui,
        }
    }
}

// ----------------------------------------------------------------------------
//pub trait WidgetApp2: eframe::App {
pub trait WidgetApp2 {
    fn fetch_watch_list(&mut self, watch_list: &Vec<crate::debugging_tools::VariableInfo>);
    // 既にeframe::Appに含まれているため、この行は不要です
    fn update(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame);
}

// ----------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Anchor {
    MonitorTab,
    SymbolPickupTab,
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<Anchor> for egui::WidgetText {
    fn from(value: Anchor) -> Self {
        Self::RichText(egui::RichText::new(value.to_string()))
    }
}

impl Default for Anchor {
    fn default() -> Self {
        Self::MonitorTab
    }
}

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
#[must_use]
enum Command {
    Nothing,
    ResetEverything,
}
// ----------------------------------------------------------------------------

/// The state that we persist (serialize).
pub struct State {
    monitor_tab: Box<dyn WidgetApp2>,
    select_tab: WatchSymbolSelectTab,

    selected_anchor: Anchor,
}

impl State {
    pub fn new(wiget_ui: Box<dyn WidgetApp2>) -> Self {
        Self {
            select_tab: Default::default(),
            monitor_tab: wiget_ui,
            selected_anchor: Default::default(),
        }
    }
}

pub struct WidgetWindow {
    pub name: String,
    pub id: u32,
    pub rect: Rect,

    state: State,
    pre_name: String,
}
//#[cfg(disable)]
impl WidgetWindow {
    //pub fn new(cc: &eframe::CreationContext<'_>, wiget_ui: Box<dyn WidgetApp2>) -> Self {
    pub fn new(id: u32, name: String, wiget_ui: Box<dyn WidgetApp2>) -> Self {
        #[allow(unused_mut)]
        let mut slf = Self {
            id,
            pre_name: name.clone(),
            name,
            state: State::new(wiget_ui),
            rect: Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
        };

        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            if let Some(state) = eframe::get_value(storage, eframe::APP_KEY) {
                slf.state = state;
            }
        }
        slf
    }

    //fn apps_iter_mut(&mut self) -> impl Iterator<Item = (&str, Anchor, &mut dyn eframe::App)> {
    fn apps_iter_mut(&mut self) -> impl Iterator<Item = (&str, Anchor)> {
        let vec = vec![
            ("📈 Monitor", Anchor::MonitorTab),
            ("📝 Watch List", Anchor::SymbolPickupTab),
        ];
        vec.into_iter()
    }

    fn update_select_tab(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.state.select_tab.update(ui, frame, self.rect);
    }

    fn update_monitor_tab(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.state.monitor_tab.update(ui, frame);
    }

    pub fn show_selected_app(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let anchor_to_update = self.state.selected_anchor;
        match anchor_to_update {
            Anchor::MonitorTab => self.update_monitor_tab(ui, frame),
            Anchor::SymbolPickupTab => self.update_select_tab(ui, frame),
        }
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cmd: &mut Command) {
        let mut selected_anchor = self.state.selected_anchor;
        for (name, anchor) in self.apps_iter_mut() {
            if ui
                .selectable_label(selected_anchor == anchor, name)
                .clicked()
            {
                selected_anchor = anchor;
                if frame.is_web() {
                    ui.ctx()
                        .open_url(egui::OpenUrl::same_tab(format!("#{anchor}")));
                }
            }
        }
        self.state.selected_anchor = selected_anchor;
    }
}
//impl eframe::App for WidgetWindow {
impl WidgetWindow {
    #[cfg(disable)]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    pub fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let now_name = self.name.clone();

        let mut wind = egui::Window::new(now_name.clone());

        if now_name != self.pre_name {
            wind = wind.current_pos(self.rect.left_top());
            self.pre_name = now_name;
        }
        let mut res = wind.show(ctx, |ui| {
            //#[cfg(disable)]
            #[cfg(not(target_arch = "wasm32"))]
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
                frame.set_fullscreen(!frame.info().window_info.fullscreen);
            }

            let mut cmd = Command::Nothing;
            egui::TopBottomPanel::top(format!("bar_{}", self.id)).show_inside(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.visuals_mut().button_frame = false;
                    self.bar_contents(ui, frame, &mut cmd);
                });
            });

            egui::TopBottomPanel::bottom(format!("btm_{}", self.id))
                .resizable(false)
                .min_height(0.0)
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {});
                });

            self.show_selected_app(ui, frame); // ctxをuiに変更

            // On web, the browser controls `pixels_per_point`.
            #[cfg(disable)]
            if !frame.is_web() {
                egui::gui_zoom::zoom_with_keyboard_shortcuts(
                    ui,
                    frame.info().native_pixels_per_point,
                ); // ctxをuiに変更
            }
        });

        if let Some(inner_response) = res {
            let rect = inner_response.response.rect;
            self.rect = rect;
        }
    }

    pub fn fetch_watch_list(&mut self, list: &Vec<VariableInfo>) {
        let new_list: Vec<SelectableVariableInfo> = list.iter()
            .map(|vals| SelectableVariableInfo {
                name: vals.name.clone(),
                types: vals.types.clone(),
                address: vals.address.clone(),
                size: vals.size,
                is_selected: false,
            })
            .collect();
    
        self.state.select_tab.watch_list.retain(|item| new_list.iter().any(|new_item| new_item.name == item.name));
        for new_item in new_list {
            if !self.state.select_tab.watch_list.iter().any(|item| item.name == new_item.name) {
                self.state.select_tab.watch_list.push(new_item);
            }
        }
    }
}

#[cfg(disable)]
pub fn set_value<T: serde::Serialize>(storage: &mut dyn Storage, key: &str, value: &T) {
    crate::profile_function!(key);
    match ron::ser::to_string(value) {
        Ok(string) => storage.set_string(key, string),
        Err(err) => log::error!("eframe failed to encode data using ron: {}", err),
    }
}

/// [`Storage`] key used for app
pub const APP_KEY: &str = "app";

// ----------------------------------------------------------------------------

#[derive(Default)]
pub struct WatchSymbolSelectTab {
    pub watch_list: Vec<SelectableVariableInfo>,
}

impl WatchSymbolSelectTab {
    #[cfg(disable)]
    pub fn fetch_watch_list(&mut self, src_list: &Vec<crate::debugging_tools::VariableInfo>) {
        self.watch_list = src_list
            .iter()
            .map(|val| SelectableVariableInfo {
                name: val.name.clone(),
                types: val.types.clone(),
                address: val.address.clone(),
                size: val.size.clone(),
                is_selected: false,
            })
            .collect();
    }

    pub fn update(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, rect:Rect) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            const CHECK_CLM:f32 = 15.;
            const TYPE_CLM:f32  = 100.;

            TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .vscroll(true)
            .drag_to_scroll(true)
            //.max_scroll_height(10.)
            .column(Column::initial(CHECK_CLM).resizable(false))
            .column(Column::initial(TYPE_CLM).resizable(true))
            .column(
                Column::initial(rect.width() - (CHECK_CLM+TYPE_CLM+50.0))
                    .at_least(50.0)
                    .resizable(true),
            )
            .header(9.0, |mut header| {
                header.col(|_| {});
                header.col(|ui| {
                    ui.heading("Type");
                });
                header.col(|ui| {
                    ui.heading("Symbol Name");
                });
            })
            .body(|mut body| {
                for selected in self.watch_list.iter_mut() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut selected.is_selected, "")
                                    .on_hover_text("add watch list");
                            });
                            row.col(|ui| {
                                ui.label(&selected.types);
                            });
                            row.col(|ui| {
                                ui.label(&selected.name).on_hover_text(&selected.name);
                            });
                        });
                    }
                });
            });
    }
}

// ----------------------------------------------------------------------------
