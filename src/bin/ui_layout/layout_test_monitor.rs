#[cfg(feature = "persistence")]
use crate::monitor_ui::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Anchor {
    SettingTab,
    MainMonitorTab,
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
        Self::SettingTab
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
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct State {
    setting_tab: SettingTab,
    main_tab: MainMonitorTab,

    selected_anchor: Anchor,
}
pub struct LayoutTest {
    state: State,
}

use egui_modal::Modal;

impl LayoutTest {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This gives us image support:
        egui_extras::install_image_loaders(&cc.egui_ctx);

        #[allow(unused_mut)]
        let mut slf = Self {
            state: State::default(),
        };

        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            if let Some(state) = eframe::get_value(storage, eframe::APP_KEY) {
                slf.state = state;
            }
        }

        slf
    }

    fn apps_iter_mut(&mut self) -> impl Iterator<Item = (&str, Anchor, &mut dyn eframe::App)> {
        #[warn(unused_mut)]
        let vec = vec![
            (
                "🔧 Setting",
                Anchor::SettingTab,
                &mut self.state.setting_tab as &mut dyn eframe::App,
            ),
            (
                "📈 Main Monitor",
                Anchor::MainMonitorTab,
                &mut self.state.main_tab as &mut dyn eframe::App,
            ),
        ];

        vec.into_iter()
    }

    fn show_selected_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let selected_anchor = self.state.selected_anchor;
        for (_name, anchor, app) in self.apps_iter_mut() {
            if anchor == selected_anchor || ctx.memory(|mem| mem.everything_is_visible()) {
                app.update(ctx, frame);
            }
        }
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cmd: &mut Command) {
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.separator();

        let mut switch_to_main_flag = false;
        let mut selected_anchor = self.state.selected_anchor;
        for (name, anchor, _app) in self.apps_iter_mut() {
            if ui
                .selectable_label(selected_anchor == anchor, name)
                .clicked()
            {
                selected_anchor = anchor;
                if frame.is_web() {
                    ui.ctx()
                        .open_url(egui::OpenUrl::same_tab(format!("#{anchor}")));
                }

                // change one shot
                if selected_anchor == Anchor::MainMonitorTab {
                    switch_to_main_flag = true;
                }
            }
        }
        self.state.selected_anchor = selected_anchor;

        if switch_to_main_flag {
            let setting = self.state.setting_tab.get_watch_setting().clone();
            self.state.main_tab.probe_if.set_probe(setting).unwrap();
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Save layout").clicked() {}

            if ui.button("Loat layout").clicked() {}

            if ui.button("Reset All layout").clicked() {
                *cmd = Command::ResetEverything;
                ui.close_menu();
            }
        });
    }

    fn reset_dialog(&mut self, ctx: &egui::Context) {
        let modal = Modal::new(ctx, "my_modal");

        // What goes inside the modal
        modal.show(|ui| {
            // these helper functions help set the ui based on the modal's
            // set style, but they are not required and you can put whatever
            // ui you want inside [`.show()`]
            modal.title(ui, "Hello world!");
            modal.frame(ui, |ui| {
                modal.body(ui, "This is a modal.");
            });
            modal.buttons(ui, |ui| {
                // After clicking, the modal is automatically closed
                if modal.button(ui, "close").clicked() {
                    println!("Hello world!")
                };
            });
        });

        modal.open();
    }

    fn run_cmd(&mut self, ctx: &egui::Context, cmd: Command) {
        match cmd {
            Command::Nothing => {}
            Command::ResetEverything => {
                self.state = Default::default();
                ctx.memory_mut(|mem| *mem = Default::default());
            }
        }
    }
}

impl eframe::App for LayoutTest {
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();

        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
            let fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
        }

        let mut cmd = Command::Nothing;
        egui::TopBottomPanel::top("app_top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                self.bar_contents(ui, frame, &mut cmd);
            });
        });

        self.show_selected_app(ctx, frame);

        self.run_cmd(ctx, cmd);
    }
}

#[cfg(feature = "ron")]
pub fn set_value<T: serde::Serialize>(storage: &mut dyn eframe::Storage, key: &str, value: &T) {
    crate::profile_function!(key);
    match ron::ser::to_string(value) {
        Ok(string) => storage.set_string(key, string),
        Err(err) => log::error!("eframe failed to encode data using ron: {}", err),
    }
}

#[allow(dead_code)]
/// [`Storage`] key used for app
pub const APP_KEY: &str = "app";
