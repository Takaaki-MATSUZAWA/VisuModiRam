mod button;
mod edit_table;
mod gauge;
mod graph_monitor;
mod slider;
mod table_view;
mod toggle_switch;
//mod widget_test;

pub use button::PushButton;
pub use edit_table::EditTable;
pub use gauge::Gauges;
pub use graph_monitor::GraphMonitor;
pub use slider::Sliders;
pub use table_view::TableView;
pub use toggle_switch::{toggle, ToggleSwitch};
//pub use widget_test::WidgetTest;
// ----------------------------------------------------------------------------
use eframe::egui::{self, LayerId, Pos2, Rect, Vec2};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

use crate::debugging_tools::*;
// ----------------------------------------------------------------------------
#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct MCUinterface {
    watch_list: Vec<VariableInfo>,

    #[cfg_attr(feature = "serde", serde(skip))]
    probe: Option<Box<ProbeInterface>>, // Boxを使用して所有権を保持
}

impl MCUinterface {
    fn fetch_watch_list(&mut self, watch_list: &Vec<crate::debugging_tools::VariableInfo>) {
        self.watch_list = watch_list.clone();
    }

    fn set_probe(&mut self, probe: ProbeInterface) {
        self.probe = Some(Box::new(probe.clone()));
    }
}
// ----------------------------------------------------------------------------
pub trait WidgetApp: serde_traitobject::Serialize + serde_traitobject::Deserialize {
    fn update(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame);

    // for MCUinterface wapper
    fn fetch_watch_list(&mut self, watch_list: &Vec<VariableInfo>);
    fn set_probe(&mut self, probe: ProbeInterface);

    // sync buttun
    fn sync_button_enable(&self) -> bool {
        false
    }

    fn disalbe_scroll_area(&self) -> bool {
        false
    }

    fn sync(&mut self) {}
}
// ----------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Anchor {
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

/// The state that we persist (serialize).
/// #[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
//#[cfg_attr(feature = "serde", serde(default))]
pub struct State {
    //#[cfg_attr(feature = "serde", serde(skip))]
    #[serde(with = "serde_traitobject")]
    monitor_tab: Box<dyn WidgetApp>,
    select_tab: WatchSymbolSelectTab,

    selected_anchor: Anchor,
}

//#[cfg(disable)]
impl Default for State {
    fn default() -> Self {
        Self {
            monitor_tab: Box::<GraphMonitor>::default(),
            select_tab: Default::default(),
            selected_anchor: Default::default(),
        }
    }
}

impl State {
    pub fn new(wiget_ui: Box<dyn WidgetApp>) -> Self {
        Self {
            monitor_tab: wiget_ui,
            select_tab: Default::default(),
            selected_anchor: Default::default(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WidgetWindow {
    pub name: String,
    pub id: u32,
    pub rect: Rect,
    pub layer_id: LayerId,

    title_bar: bool,
    lock: bool,

    state: State,
    pre_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    first_update_flag_inv: bool,
}

// basic ui functions
impl WidgetWindow {
    pub fn new(
        id: u32,
        name: String,
        title_bar: bool,
        lock: bool,
        widget_ui: Box<dyn WidgetApp>,
    ) -> Self {
        Self {
            id,
            pre_name: name.clone(),
            name,
            state: State::new(widget_ui),
            rect: Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(300.0, 400.0)),
            layer_id: LayerId::new(egui::Order::Middle, egui::Id::new(id)),
            first_update_flag_inv: true,
            title_bar,
            lock,
        }
    }

    fn apps_iter_mut(&mut self) -> impl Iterator<Item = (&str, Anchor)> {
        let vec = vec![
            ("📈 Monitor", Anchor::MonitorTab),
            ("📝 Watch List", Anchor::SymbolPickupTab),
        ];
        vec.into_iter()
    }

    fn update_select_tab(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.state.select_tab.update(ui, frame);
    }

    fn update_monitor_tab(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.state.monitor_tab.update(ui, frame);
    }

    fn show_selected_app(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let anchor_to_update = self.state.selected_anchor;
        match anchor_to_update {
            Anchor::MonitorTab => self.update_monitor_tab(ui, frame),
            Anchor::SymbolPickupTab => self.update_select_tab(ui, frame),
        }
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let is_selected = self.state.selected_anchor;
        let mut new_anchor = None;

        for (name, anchor) in self.apps_iter_mut() {
            if ui.selectable_label(is_selected == anchor, name).clicked() {
                new_anchor = Some(anchor);
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if self.state.monitor_tab.sync_button_enable() {
                if ui.button("Sync from MCU").clicked() {
                    self.state.monitor_tab.sync();
                }
            }
            ui.separator();
        });

        if let Some(anchor) = new_anchor {
            self.switch_tab_to(anchor);
        }
    }
}

// update ui function
impl WidgetWindow {
    pub fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, open: &mut bool) {
        let now_name = self.name.clone();

        let id = egui::Id::new(self.id);
        let mut wind = egui::Window::new(now_name.clone())
            .id(id)
            .title_bar(self.title_bar)
            .resizable(!self.lock)
            .movable(!self.lock)
            .default_width(380.0)
            .default_height(280.0);
        wind = wind.open(open);

        self.layer_id = egui::LayerId::new(egui::Order::Middle, id);

        if now_name != self.pre_name {
            wind = wind.current_pos(self.rect.left_top());
            self.pre_name = now_name;
        }

        if !self.first_update_flag_inv {
            //println!("{} first update", self.name);
            wind = wind.current_pos(self.rect.left_top());
            wind = wind.fixed_size(Vec2::new(self.rect.width(), self.rect.height()));
            self.first_update_flag_inv = true;
        }

        let res = wind.show(ctx, |ui| {
            #[cfg(not(target_arch = "wasm32"))]
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
                let fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
            }

            egui::TopBottomPanel::top(format!("bar_{}", self.id)).show_inside(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.visuals_mut().button_frame = false;
                    self.bar_contents(ui, frame);
                });
            });

            StripBuilder::new(ui)
                .size(Size::remainder().at_least(100.0)) // for the table
                .size(Size::exact(0.5)) // for the source code link
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        if self.state.monitor_tab.disalbe_scroll_area() {
                            ui.set_clip_rect(self.rect.clone());
                            self.show_selected_app(ui, frame); // ctxをuiに変更
                        } else {
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                self.show_selected_app(ui, frame); // ctxをuiに変更
                            });
                        }
                    });
                    strip.cell(|ui| {
                        ui.vertical_centered(|_ui| {});
                    });
                });
        });

        if let Some(inner_response) = res {
            let rect = inner_response.response.rect;
            self.rect = rect;
        }
    }
}

// public functions
impl WidgetWindow {
    pub fn fetch_watch_list(&mut self, list: &Vec<VariableInfo>) {
        SelectableVariableInfo::fetch(&list, &mut self.state.select_tab.watch_list);
    }

    pub fn set_probe_to_app(&mut self, probe: ProbeInterface) {
        self.state.monitor_tab.set_probe(probe);
    }

    pub fn switch_tab_to(&mut self, anchor: Anchor) {
        self.state.selected_anchor = anchor;

        if anchor == Anchor::MonitorTab {
            let list = SelectableVariableInfo::pick_selected(&self.state.select_tab.watch_list);
            self.state.monitor_tab.fetch_watch_list(&list.clone());
        }
    }

    pub fn show_title_bar(&mut self, enable: bool) {
        self.title_bar = enable;
    }

    pub fn lock(&mut self, enable: bool) {
        self.lock = enable;
    }
}

// ----------------------------------------------------------------------------

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WatchSymbolSelectTab {
    pub watch_list: Vec<SelectableVariableInfo>,
}

impl WatchSymbolSelectTab {
    pub fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            if ui.button("All select").clicked() {
                self.watch_list.iter_mut().for_each(|symbol| {
                    symbol.is_selected = true;
                });
            }
            if ui.button("Deselect all").clicked() {
                self.watch_list.iter_mut().for_each(|symbol| {
                    symbol.is_selected = false;
                });
            }
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            const CHECK_CLM: f32 = 15.;
            const TYPE_CLM: f32 = 100.;

            TableBuilder::new(ui)
                .striped(true)
                .min_scrolled_height(0.0)
                .column(Column::exact(CHECK_CLM).resizable(false))
                .column(Column::initial(TYPE_CLM).resizable(true))
                .column(Column::remainder())
                .header(9.0, |mut header| {
                    header.col(|_| {});
                    header.col(|ui| {
                        ui.strong("Type");
                    });
                    header.col(|ui| {
                        ui.strong("Symbol Name");
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
