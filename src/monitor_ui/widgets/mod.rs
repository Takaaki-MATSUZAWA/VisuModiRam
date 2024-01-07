mod graph_monitor;
mod widget_test;

pub use graph_monitor::GraphMonitor;
pub use widget_test::WidgetTest;
// ----------------------------------------------------------------------------
use eframe::egui::{self, Pos2, Rect, Vec2};
use egui_extras::{Column, TableBuilder};

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
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum WidgetAppKind {
    GraphMonitor,
    WidgetTest,
}

impl std::fmt::Display for WidgetAppKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
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

    state: State,
    kind: WidgetAppKind,
    pre_name: String,
}

// basic ui functions
impl WidgetWindow {
    pub fn new(id: u32, name: String, kind: WidgetAppKind) -> Self {
        let widget_ui: Box<dyn WidgetApp> = match kind {
            WidgetAppKind::GraphMonitor => Box::new(GraphMonitor::default()),
            WidgetAppKind::WidgetTest => Box::new(WidgetTest::default()),
        };

        #[allow(unused_mut)]
        let mut slf = Self {
            id,
            pre_name: name.clone(),
            name,
            state: State::new(widget_ui),
            rect: Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
            kind,
        };
        slf
    }

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

        if let Some(anchor) = new_anchor {
            self.switch_tab_to(anchor);
        }
    }
}

// update ui function
impl WidgetWindow {
    pub fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let now_name = self.name.clone();

        let mut wind = egui::Window::new(now_name.clone());

        if now_name != self.pre_name {
            wind = wind.current_pos(self.rect.left_top());
            self.pre_name = now_name;
        }
        let res = wind.show(ctx, |ui| {
            #[cfg(not(target_arch = "wasm32"))]
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
                frame.set_fullscreen(!frame.info().window_info.fullscreen);
            }

            egui::TopBottomPanel::top(format!("bar_{}", self.id)).show_inside(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.visuals_mut().button_frame = false;
                    self.bar_contents(ui, frame);
                });
            });

            egui::TopBottomPanel::bottom(format!("btm_{}", self.id))
                .resizable(false)
                .min_height(0.0)
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|_ui| {});
                });

            self.show_selected_app(ui, frame); // ctxをuiに変更
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
}

// ----------------------------------------------------------------------------

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WatchSymbolSelectTab {
    pub watch_list: Vec<SelectableVariableInfo>,
}

impl WatchSymbolSelectTab {
    pub fn update(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame, rect: Rect) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            const CHECK_CLM: f32 = 15.;
            const TYPE_CLM: f32 = 100.;

            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .vscroll(true)
                .drag_to_scroll(true)
                //.max_scroll_height(10.)
                .column(Column::initial(CHECK_CLM).resizable(false))
                .column(Column::initial(TYPE_CLM).resizable(true))
                .column(
                    Column::initial(rect.width() - (CHECK_CLM + TYPE_CLM + 50.0))
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
