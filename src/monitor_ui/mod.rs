mod main_monitor;
mod setting_panel;
mod logging_tab;
mod data_viewer_tab;

pub use main_monitor::MainMonitorTab;
pub use setting_panel::SettingTab;
pub use logging_tab::LoggingTab;
pub use data_viewer_tab::DataViewerTab;

mod widgets;
pub use widgets::*;
