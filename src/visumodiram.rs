#[cfg(feature = "persistence")]
use crate::monitor_ui::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Anchor {
    SettingTab,
    MainMonitorTab,
    LoggingTab,
    DataViewerTab,
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
#[derive(Clone, Copy, Debug)]
#[must_use]
enum Dialog {
    None,
    Reset,
    FaildLoadSaveData,
}
// ----------------------------------------------------------------------------

/// The state that we persist (serialize).
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct State {
    setting_tab: SettingTab,
    main_tab: MainMonitorTab,
    logging_tab: LoggingTab,
    data_viewer_tab: DataViewerTab,

    selected_anchor: Anchor,
}
pub struct VisuModiRam {
    state: State,

    open_dialog: Dialog,
}

// use egui_modal::Modal;  // Temporarily disabled
use rfd::FileDialog;
use std::path::PathBuf;

impl VisuModiRam {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This gives us image support:
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "Inter".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/Inter-Regular.otf")).into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Inter".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        #[allow(unused_mut)]
        let mut slf = Self {
            state: State::default(),
            open_dialog: Dialog::None,
        };

        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            if let Some(mut state) = eframe::get_value::<State>(storage, eframe::APP_KEY) {
                // Restore widgets from serialized configs
                state.main_tab.restore_from_load();
                // Always start on Setting tab
                state.selected_anchor = Anchor::SettingTab;
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
            (
                "🚀 High-Speed Logging",
                Anchor::LoggingTab,
                &mut self.state.logging_tab as &mut dyn eframe::App,
            ),
            (
                "📊 Data Viewer",
                Anchor::DataViewerTab,
                &mut self.state.data_viewer_tab as &mut dyn eframe::App,
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

                // change one shot - probe settings transfer
                if selected_anchor == Anchor::MainMonitorTab || selected_anchor == Anchor::LoggingTab {
                    switch_to_main_flag = true;
                }
            }
        }
        self.state.selected_anchor = selected_anchor;

        if switch_to_main_flag {
            let setting = self.state.setting_tab.get_watch_setting().clone();
            if selected_anchor == Anchor::MainMonitorTab {
                self.state.main_tab.probe_if.set_probe(setting).unwrap();
            } else if selected_anchor == Anchor::LoggingTab {
                let mut probe_if = crate::debugging_tools::ProbeInterface::default();
                probe_if.set_probe(setting).unwrap();
                self.state.logging_tab.set_probe(probe_if).unwrap();
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Reset All layout").clicked() {
                self.open_dialog = Dialog::Reset;
            }
            ui.separator();
            if ui.button("Load layout").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("Layout file", &["ron"])
                    .pick_file()
                {
                    if let Ok(mut load_data) = self::load_layout::<State>(path) {
                        // Restore widgets from serialized configs
                        load_data.main_tab.restore_from_load();
                        self.state = load_data;
                    } else {
                        //println!("faild load layout");
                        self.open_dialog = Dialog::FaildLoadSaveData;
                    }
                }
            }
            ui.separator();
            if ui.button("Save layout").clicked() {
                if let Some(path) = FileDialog::new()
                    .set_file_name("monitor_layout")
                    .add_filter("Layout file", &["ron"])
                    .save_file()
                {
                    //println!("{:?}", path);
                    let mut path_with_extension = path.clone();
                    if !path.to_str().map_or(false, |s| s.ends_with(".ron")) {
                        path_with_extension = path.with_extension("ron");
                    }
                    // Prepare widget configs for serialization
                    self.state.main_tab.prepare_for_save();
                    self::save_layout(path_with_extension, &self.state);
                }
            }
            ui.separator();
        });

        match self.open_dialog {
            Dialog::Reset => self.reset_dialog_ui(ui.ctx(), cmd),
            Dialog::FaildLoadSaveData => self.faild_load_save_data_dialog_ui(ui.ctx(), cmd),
            _ => {}
        }
    }

    fn reset_dialog_ui(&mut self, ctx: &egui::Context, cmd: &mut Command) {
        egui::Window::new("Warning!")
            .open(&mut true)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Are you sure you want to RESET ALL layouts, elf file paths and watchlists?");
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.open_dialog = Dialog::None;
                    }
                    if ui.button("All Reset").clicked() {
                        *cmd = Command::ResetEverything;
                        self.open_dialog = Dialog::None;
                    }
                });
            });
    }

    fn faild_load_save_data_dialog_ui(&mut self, ctx: &egui::Context, _cmd: &mut Command) {
        egui::Window::new("Error!")
            .open(&mut true)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Failed to load layout save data.");
                if ui.button("Accept").clicked() {
                    self.open_dialog = Dialog::None;
                }
            });
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

impl eframe::App for VisuModiRam {
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Prepare widget configurations for serialization
        self.state.main_tab.prepare_for_save();
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Note: on_exit doesn't have access to storage, so we rely on the save() method
        // which is called automatically before on_exit
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        // Auto-save every 30 seconds
        std::time::Duration::from_secs(30)
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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

pub fn save_layout<T: serde::Serialize>(save_file: PathBuf, value: &T) {
    let serialized = ron::ser::to_string(&value).expect("Failed to serialize state");
    //println!("serialized!!!");
    std::fs::write(save_file, serialized).expect("Failed to write to file");
    //println!("saved!!!");
}

pub fn load_layout<T: serde::de::DeserializeOwned>(load_file: PathBuf) -> Result<T, String> {
    let serialized_data = std::fs::read_to_string(load_file)
        .map_err(|err| format!("Failed to read from file: {}", err))?;
    let deserialized = ron::de::from_str(&serialized_data)
        .map_err(|err| format!("Failed to deserialize state: {}", err))?;
    Ok(deserialized)
}
