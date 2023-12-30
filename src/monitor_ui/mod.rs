mod main_monitor;
mod probe_setting;
mod setting_panel;
mod symbol_search;

pub use main_monitor::MainMonitorTab;
pub use probe_setting::ProbeSetting;
pub use setting_panel::SettingTab;
pub use symbol_search::SymbolSearch;

mod setting_panel2;
pub use setting_panel2::SettingTab2;

mod widgets;
pub use widgets::*;
