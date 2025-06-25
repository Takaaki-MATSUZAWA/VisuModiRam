use eframe::egui::{self, Color32, RichText};
use egui_plot::{Line, Plot, PlotPoints};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::debugging_tools::ProbeInterface;

fn default_display_count() -> usize {
    100
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LoggingState {
    Stopped,
    Running,
    Paused,
}

impl Default for LoggingState {
    fn default() -> Self {
        Self::Stopped
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggingSettings {
    pub sample_rate_ms: u64,       // サンプリング間隔（ミリ秒）
    pub buffer_size_mb: usize,     // バッファサイズ（MB）
    pub auto_save_interval_s: u64, // 自動保存間隔（秒）
    pub max_log_duration_s: u64,   // 最大ログ時間（秒、0で無制限）
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            sample_rate_ms: 10,       // 10ms間隔で高速ロギング
            buffer_size_mb: 100,      // 100MBバッファ
            auto_save_interval_s: 30, // 30秒間隔で自動保存
            max_log_duration_s: 0,    // 無制限
        }
    }
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LoggingTab {
    pub probe_if: ProbeInterface,

    #[cfg_attr(feature = "serde", serde(skip))]
    logging_state: LoggingState,

    settings: LoggingSettings,

    #[cfg_attr(feature = "serde", serde(skip))]
    start_time: Option<Instant>,

    #[cfg_attr(feature = "serde", serde(skip))]
    last_save_time: Option<Instant>,

    #[cfg_attr(feature = "serde", serde(skip))]
    total_samples: u64,

    #[cfg_attr(feature = "serde", serde(skip))]
    current_log_size_mb: f64,

    // ログデータ表示用
    selected_variables: Vec<String>,
    #[cfg_attr(feature = "serde", serde(default = "default_display_count"))]
    display_data_count: usize,

    // プロット表示用
    plot_visible_variables: Vec<String>,
    plot_auto_range: bool,

    // 統計情報
    #[cfg_attr(feature = "serde", serde(skip))]
    samples_per_second: f64,

    // エラー情報
    #[cfg_attr(feature = "serde", serde(skip))]
    last_error: Option<String>,
}

impl eframe::App for LoggingTab {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // リアルタイム更新のリクエスト
        if matches!(self.logging_state, LoggingState::Running) {
            ctx.request_repaint();
            self.update_logging_stats();
        }

        egui::SidePanel::left("logging_control")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                self.logging_control_ui(ui);
            });

        egui::SidePanel::right("logging_stats")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                self.logging_stats_ui(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // データ表示部分を上下に分割
            ui.horizontal(|ui| {
                ui.set_height(ctx.available_rect().height());

                ui.allocate_ui_with_layout(
                    [
                        ctx.available_rect().width() * 0.6,
                        ctx.available_rect().height(),
                    ]
                    .into(),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.plot_display_ui(ui);
                    },
                );
            });
        });
    }
}

impl LoggingTab {
    pub fn set_probe(&mut self, probe_if: ProbeInterface) -> Result<(), String> {
        self.probe_if = probe_if;
        Ok(())
    }

    fn logging_control_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚀 High-Speed Logging");
        ui.separator();

        // プローブ接続状態
        ui.horizontal(|ui| {
            let is_connected = !self.probe_if.setting.probe_sn.is_empty()
                && !self.probe_if.setting.target_mcu.is_empty();

            if is_connected {
                ui.label(RichText::new("🟢 Probe Connected").color(Color32::GREEN));
            } else {
                ui.label(RichText::new("🔴 Probe Disconnected").color(Color32::RED));
            }
        });

        ui.label(format!("MCU: {}", self.probe_if.setting.target_mcu));
        ui.label(format!(
            "Variables: {}",
            self.probe_if.setting.watch_list.len()
        ));

        ui.separator();

        // ロギング設定
        ui.heading("Settings");

        ui.horizontal(|ui| {
            ui.label("Sample Rate:");
            ui.add(
                egui::DragValue::new(&mut self.settings.sample_rate_ms)
                    .suffix(" ms")
                    .range(1..=1000)
                    .speed(1.0),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Buffer Size:");
            ui.add(
                egui::DragValue::new(&mut self.settings.buffer_size_mb)
                    .suffix(" MB")
                    .range(10..=1000)
                    .speed(10.0),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Auto Save:");
            ui.add(
                egui::DragValue::new(&mut self.settings.auto_save_interval_s)
                    .suffix(" sec")
                    .range(5..=300)
                    .speed(5.0),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Max Duration:");
            ui.add(
                egui::DragValue::new(&mut self.settings.max_log_duration_s)
                    .suffix(" sec (0=∞)")
                    .range(0..=3600)
                    .speed(10.0),
            );
        });

        ui.separator();

        // ロギング制御ボタン
        let is_connected = !self.probe_if.setting.probe_sn.is_empty();
        let can_start = is_connected
            && matches!(
                self.logging_state,
                LoggingState::Stopped | LoggingState::Paused
            );
        let can_pause = matches!(self.logging_state, LoggingState::Running);
        let can_stop = matches!(
            self.logging_state,
            LoggingState::Running | LoggingState::Paused
        );

        ui.horizontal(|ui| {
            if ui
                .add_enabled(can_start, egui::Button::new("▶️ Start"))
                .clicked()
            {
                self.start_logging();
            }

            if ui
                .add_enabled(can_pause, egui::Button::new("⏸️ Pause"))
                .clicked()
            {
                self.pause_logging();
            }

            if ui
                .add_enabled(can_stop, egui::Button::new("⏹️ Stop"))
                .clicked()
            {
                self.stop_logging();
            }
        });

        // 状態表示
        let state_text = match self.logging_state {
            LoggingState::Stopped => "⏹️ Stopped",
            LoggingState::Running => "▶️ Running",
            LoggingState::Paused => "⏸️ Paused",
        };

        let state_color = match self.logging_state {
            LoggingState::Stopped => Color32::GRAY,
            LoggingState::Running => Color32::GREEN,
            LoggingState::Paused => Color32::YELLOW,
        };

        ui.label(RichText::new(state_text).color(state_color));

        // エラー表示
        if let Some(error) = &self.last_error {
            ui.separator();
            ui.label(RichText::new(format!("⚠️ Error: {}", error)).color(Color32::RED));
        }

        ui.separator();

        // 保存制御
        ui.heading("Data Export");

        ui.horizontal(|ui| {
            if ui.button("💾 Save Binary").clicked() {
                self.save_data_binary();
            }
            if ui.button("📄 Save CSV").clicked() {
                self.save_data_csv();
            }
        });

        if ui.button("🗑️ Clear Data").clicked() {
            self.clear_data();
        }
    }

    fn logging_stats_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Statistics");
        ui.separator();

        // 実行時間
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            ui.label(format!(
                "Duration: {:02}:{:02}:{:02}",
                elapsed.as_secs() / 3600,
                (elapsed.as_secs() % 3600) / 60,
                elapsed.as_secs() % 60
            ));
        } else {
            ui.label("Duration: --:--:--");
        }

        // サンプル統計
        ui.label(format!("Total Samples: {}", self.total_samples));
        ui.label(format!("Samples/sec: {:.1}", self.samples_per_second));
        ui.label(format!("Buffer Usage: {:.1} MB", self.current_log_size_mb));

        // プログレスバー（バッファ使用量）
        let buffer_usage =
            (self.current_log_size_mb / self.settings.buffer_size_mb as f64).clamp(0.0, 1.0);
        ui.add(
            egui::ProgressBar::new(buffer_usage as f32)
                .text(format!("{:.1}% buffer used", buffer_usage * 100.0)),
        );

        ui.separator();

        // 変数選択
        ui.heading("📋 Variables");

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for variable in &self.probe_if.setting.watch_list {
                    let mut is_selected = self.selected_variables.contains(&variable.name);
                    if ui.checkbox(&mut is_selected, &variable.name).changed() {
                        if is_selected {
                            if !self.selected_variables.contains(&variable.name) {
                                self.selected_variables.push(variable.name.clone());
                            }
                        } else {
                            self.selected_variables.retain(|x| x != &variable.name);
                        }
                    }
                }
            });

        // 全選択/全解除
        ui.horizontal(|ui| {
            if ui.button("Select All").clicked() {
                self.selected_variables = self
                    .probe_if
                    .setting
                    .watch_list
                    .iter()
                    .map(|v| v.name.clone())
                    .collect();
            }
            if ui.button("Clear All").clicked() {
                self.selected_variables.clear();
            }
        });
    }

    fn plot_display_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Data Plot");
        ui.separator();

        // プロット制御
        let mut reset_flag = false;
        ui.horizontal(|ui| {
            if ui.button("Pos Reset").clicked() {
                reset_flag = true;
            }
            ui.separator();
            ui.label("Plot Variables:");
            ui.checkbox(&mut self.plot_auto_range, "Auto Range");
        });

        // 変数選択（ログ終了時のみ有効）
        let logging_stopped = matches!(self.logging_state, LoggingState::Stopped);
        ui.add_enabled_ui(logging_stopped, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Show:");
                for var_info in &self.probe_if.setting.watch_list {
                    let mut is_visible = self.plot_visible_variables.contains(&var_info.name);
                    if ui.checkbox(&mut is_visible, &var_info.name).changed() {
                        if is_visible {
                            if !self.plot_visible_variables.contains(&var_info.name) {
                                self.plot_visible_variables.push(var_info.name.clone());
                            }
                        } else {
                            self.plot_visible_variables.retain(|x| x != &var_info.name);
                        }
                    }
                }
            });
        });

        if !logging_stopped {
            ui.label(
                RichText::new("🔴 Plot is available after logging stops").color(Color32::YELLOW),
            );
            return;
        }

        if self.plot_visible_variables.is_empty() {
            ui.label("Select variables to plot");
            return;
        }

        ui.separator();

        // データ取得とプロット表示
        let plot_data = self.get_plot_data();

        if plot_data.is_empty() {
            ui.label("No data available for plotting");
            return;
        }

        // プロット描画
        let mut plot = Plot::new("logging_plot")
            .legend(egui_plot::Legend::default())
            .height(400.0)
            .allow_zoom(true)
            .allow_drag(true)
            .allow_scroll(true);

        if self.plot_auto_range {
            plot = plot.auto_bounds([true, true].into());
        }

        if reset_flag {
            plot = plot.reset();
        }

        plot.show(ui, |plot_ui| {
            // 各変数のラインを描画
            for (var_name, data) in &plot_data {
                if !data.is_empty() {
                    let points: PlotPoints =
                        data.iter().map(|[time, value]| [*time, *value]).collect();

                    let line = Line::new(points).name(var_name).width(2.0);

                    plot_ui.line(line);
                }
            }
        });

        // データ統計情報
        ui.separator();
        ui.collapsing("📈 Plot Statistics", |ui| {
            for (var_name, data) in &plot_data {
                if !data.is_empty() {
                    let values: Vec<f64> = data.iter().map(|[_, value]| *value).collect();
                    let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                    let avg_val = values.iter().sum::<f64>() / values.len() as f64;

                    ui.label(format!(
                        "{}: Min={:.3}, Max={:.3}, Avg={:.3}, Samples={}",
                        var_name,
                        min_val,
                        max_val,
                        avg_val,
                        data.len()
                    ));
                }
            }
        });
    }

    fn get_plot_data(&mut self) -> std::collections::HashMap<String, Vec<[f64; 2]>> {
        let mut plot_data = std::collections::HashMap::new();

        // 大きな時間窓でデータを取得（全データ）
        let large_time_window = 3600000; // 1時間分

        for var_name in &self.plot_visible_variables {
            let data = self.probe_if.get_log_vec(var_name, Some(large_time_window));
            if !data.is_empty() {
                plot_data.insert(var_name.clone(), data);
            }
        }

        plot_data
    }

    fn start_logging(&mut self) {
        if !self.probe_if.setting.probe_sn.is_empty() {
            self.logging_state = LoggingState::Running;
            self.start_time = Some(Instant::now());
            self.last_save_time = Some(Instant::now());
            self.last_error = None;

            // probe_ifでのロギング開始
            self.probe_if
                .watching_start(Duration::from_millis(self.settings.sample_rate_ms));
        } else {
            self.last_error = Some("No probe connected".to_string());
        }
    }

    fn pause_logging(&mut self) {
        self.logging_state = LoggingState::Paused;
        self.probe_if.watching_stop();
    }

    fn stop_logging(&mut self) {
        self.logging_state = LoggingState::Stopped;
        self.probe_if.watching_stop();
        self.start_time = None;
    }

    fn update_logging_stats(&mut self) {
        // 実際のsensorlog-ramからメモリ使用量を取得
        self.current_log_size_mb = self.probe_if.get_memory_usage() as f64 / (1024.0 * 1024.0);

        // 各センサーからの総サンプル数を取得
        let mut total_measurements = 0;
        let watch_list = self.probe_if.setting.watch_list.clone();
        for var_info in &watch_list {
            total_measurements += self.probe_if.get_measurement_count(&var_info.name);
        }
        self.total_samples = total_measurements as u64;

        // サンプリングレート計算
        if let Some(start_time) = self.start_time {
            let elapsed_seconds = start_time.elapsed().as_secs_f64();
            if elapsed_seconds > 0.0 && self.total_samples > 0 {
                self.samples_per_second = self.total_samples as f64 / elapsed_seconds;
            }
        }

        // 自動保存チェック
        if let Some(last_save) = self.last_save_time {
            if last_save.elapsed().as_secs() >= self.settings.auto_save_interval_s {
                self.auto_save_data();
                self.last_save_time = Some(Instant::now());
            }
        }

        // 最大時間チェック
        if self.settings.max_log_duration_s > 0 {
            if let Some(start_time) = self.start_time {
                if start_time.elapsed().as_secs() >= self.settings.max_log_duration_s {
                    self.stop_logging();
                }
            }
        }

        // バッファ容量チェック
        if self.current_log_size_mb >= self.settings.buffer_size_mb as f64 {
            self.last_error = Some(format!(
                "Buffer full ({:.1} MB). Stopping logging.",
                self.current_log_size_mb
            ));
            self.stop_logging();
        }
    }

    fn save_data_binary(&mut self) {
        use rfd::FileDialog;

        if let Some(_path) = FileDialog::new()
            .set_file_name("high_speed_log")
            .add_filter("Binary data", &["bin"])
            .save_file()
        {
            match self.probe_if.force_save_to_disk() {
                Ok(_) => {
                    self.last_error = None;
                    // プラットフォーム固有の通知（将来の拡張）
                }
                Err(e) => {
                    self.last_error = Some(format!("Binary save failed: {}", e));
                }
            }
        }
    }

    fn save_data_csv(&mut self) {
        use rfd::FileDialog;

        if let Some(path) = FileDialog::new()
            .set_file_name("high_speed_log")
            .add_filter("CSV file", &["csv"])
            .save_file()
        {
            match self.export_to_csv(path) {
                Ok(_) => {
                    self.last_error = None;
                }
                Err(e) => {
                    self.last_error = Some(format!("CSV export failed: {}", e));
                }
            }
        }
    }

    fn export_to_csv(&mut self, path: std::path::PathBuf) -> Result<(), std::io::Error> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;

        // ヘッダー書き込み
        write!(file, "Timestamp")?;
        for var_name in &self.probe_if.setting.watch_list {
            write!(file, ",{}", var_name.name)?;
        }
        writeln!(file)?;

        // データの取得と書き込み
        let export_count = 10000; // 最大エクスポート数
        let time_window = export_count as u64 * self.settings.sample_rate_ms;

        // 全変数のデータを取得
        let mut all_data = std::collections::HashMap::new();
        let mut max_len = 0;

        let watch_list = self.probe_if.setting.watch_list.clone();
        for var_info in &watch_list {
            let data = self.probe_if.get_log_vec(&var_info.name, Some(time_window));
            max_len = max_len.max(data.len());
            all_data.insert(var_info.name.clone(), data);
        }

        // データを時系列順に整理して書き込み
        for i in 0..max_len {
            let mut row_written = false;

            // タイムスタンプを最初の変数から取得
            if let Some(first_var) = watch_list.first() {
                if let Some(data) = all_data.get(&first_var.name) {
                    if i < data.len() {
                        write!(file, "{:.6}", data[i][0])?;
                        row_written = true;
                    }
                }
            }

            if !row_written {
                write!(
                    file,
                    "{:.6}",
                    i as f64 * self.settings.sample_rate_ms as f64 / 1000.0
                )?;
            }

            // 各変数の値
            for var_info in &watch_list {
                if let Some(data) = all_data.get(&var_info.name) {
                    if i < data.len() {
                        write!(file, ",{:.6}", data[i][1])?;
                    } else {
                        write!(file, ",")?;
                    }
                } else {
                    write!(file, ",")?;
                }
            }
            writeln!(file)?;
        }

        Ok(())
    }

    fn auto_save_data(&mut self) {
        match self.probe_if.force_save_to_disk() {
            Ok(_) => {
                // 自動保存成功、エラーをクリア
                if let Some(error) = &self.last_error {
                    if error.contains("auto save") {
                        self.last_error = None;
                    }
                }
            }
            Err(e) => {
                self.last_error = Some(format!("Auto save failed: {}", e));
            }
        }
    }

    fn clear_data(&mut self) {
        // 全センサーのRAMデータをクリア
        for var_info in &self.probe_if.setting.watch_list.clone() {
            self.probe_if.clear_sensor_ram(&var_info.name);
        }

        // 統計情報をリセット
        self.total_samples = 0;
        self.current_log_size_mb = 0.0;
        self.last_error = None;
        self.start_time = None;
        self.last_save_time = None;
    }
}
