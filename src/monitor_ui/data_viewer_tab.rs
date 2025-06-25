use eframe::egui::{self, Color32, RichText};
use egui_extras::{Column, TableBuilder};
use egui_plot::{Line, Plot, PlotPoints};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

fn default_table_rows() -> usize {
    200
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataFile {
    pub path: PathBuf,
    pub name: String,
    pub variables: Vec<String>,
    pub sample_count: usize,
    pub duration: f64, // seconds
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlotSettings {
    pub visible_variables: Vec<String>,
    pub auto_range: bool,
    pub line_width: f32,
    pub show_points: bool,
    pub time_range_start: Option<f64>,
    pub time_range_end: Option<f64>,
}

impl Default for PlotSettings {
    fn default() -> Self {
        Self {
            visible_variables: Vec::new(),
            auto_range: true,
            line_width: 2.0,
            show_points: false,
            time_range_start: None,
            time_range_end: None,
        }
    }
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DataViewerTab {
    // ファイル管理
    loaded_files: Vec<DataFile>,
    selected_file_index: Option<usize>,

    // データ表示
    plot_settings: PlotSettings,
    #[cfg_attr(feature = "serde", serde(skip))]
    loaded_data: HashMap<String, Vec<[f64; 2]>>, // variable_name -> [(time, value)]

    // UI状態
    show_table: bool,
    #[cfg_attr(feature = "serde", serde(default = "default_table_rows"))]
    table_rows_to_show: usize,

    // ファイル読み込み状態
    #[cfg_attr(feature = "serde", serde(skip))]
    loading_progress: f32,
    #[cfg_attr(feature = "serde", serde(skip))]
    loading_message: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    last_error: Option<String>,
}

impl eframe::App for DataViewerTab {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("file_manager")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                self.file_manager_ui(ui);
            });

        egui::SidePanel::right("plot_controls")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                self.plot_controls_ui(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_table {
                // テーブルとプロットを上下分割
                ui.horizontal(|ui| {
                    ui.set_height(ctx.available_rect().height());

                    // 左側: データテーブル
                    ui.allocate_ui_with_layout(
                        [
                            ctx.available_rect().width() * 0.4,
                            ctx.available_rect().height(),
                        ]
                        .into(),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            self.data_table_ui(ui);
                        },
                    );

                    ui.separator();

                    // 右側: プロット
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
            } else {
                // プロットのみ
                self.plot_display_ui(ui);
            }
        });
    }
}

impl DataViewerTab {
    fn file_manager_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("📁 Data Files");
        ui.separator();

        // ファイル読み込みボタン
        ui.horizontal(|ui| {
            if ui.button("📂 Load CSV File").clicked() {
                self.load_csv_file();
            }
            if ui.button("🗑️ Clear All").clicked() {
                self.clear_all_data();
            }
        });

        // 読み込み進捗
        if self.loading_progress > 0.0 && self.loading_progress < 1.0 {
            ui.add(
                egui::ProgressBar::new(self.loading_progress)
                    .text(&self.loading_message)
                    .animate(true),
            );
        }

        // エラー表示
        if let Some(error) = &self.last_error {
            ui.label(RichText::new(format!("❌ Error: {}", error)).color(Color32::RED));
        }

        ui.separator();

        // ファイルリスト
        ui.heading("Loaded Files");

        let mut file_to_remove = None;
        let mut file_to_select = None;

        for (index, data_file) in self.loaded_files.iter().enumerate() {
            ui.horizontal(|ui| {
                // ファイル選択ラジオボタン
                let is_selected = self.selected_file_index == Some(index);
                if ui.radio(is_selected, "").clicked() {
                    file_to_select = Some(index);
                }

                // ファイル情報
                ui.vertical(|ui| {
                    ui.label(RichText::new(&data_file.name).strong());
                    ui.label(format!("Variables: {}", data_file.variables.len()));
                    ui.label(format!("Samples: {}", data_file.sample_count));
                    ui.label(format!("Duration: {:.2}s", data_file.duration));
                });

                // 削除ボタン
                if ui.button("🗑️").clicked() {
                    file_to_remove = Some(index);
                }
            });
            ui.separator();
        }

        // ファイル選択処理
        if let Some(index) = file_to_select {
            self.selected_file_index = Some(index);
            self.load_file_data(index);
        }

        // ファイル削除処理
        if let Some(index) = file_to_remove {
            self.loaded_files.remove(index);
            if self.selected_file_index == Some(index) {
                self.selected_file_index = None;
                self.loaded_data.clear();
            } else if let Some(selected) = self.selected_file_index {
                if selected > index {
                    self.selected_file_index = Some(selected - 1);
                }
            }
        }

        if self.loaded_files.is_empty() {
            ui.label("No files loaded");
        }
    }

    fn plot_controls_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎛️ Plot Controls");
        ui.separator();

        // 基本設定
        ui.checkbox(&mut self.show_table, "Show Data Table");

        ui.separator();

        // 時間範囲設定
        ui.heading("Time Range");
        ui.horizontal(|ui| {
            ui.label("Start:");
            let mut start_enabled = self.plot_settings.time_range_start.is_some();
            if ui.checkbox(&mut start_enabled, "").changed() {
                self.plot_settings.time_range_start = if start_enabled { Some(0.0) } else { None };
            }
            if let Some(ref mut start) = self.plot_settings.time_range_start {
                ui.add(egui::DragValue::new(start).suffix("s").speed(0.1));
            }
        });

        ui.horizontal(|ui| {
            ui.label("End:");
            let mut end_enabled = self.plot_settings.time_range_end.is_some();
            if ui.checkbox(&mut end_enabled, "").changed() {
                self.plot_settings.time_range_end = if end_enabled { Some(10.0) } else { None };
            }
            if let Some(ref mut end) = self.plot_settings.time_range_end {
                ui.add(egui::DragValue::new(end).suffix("s").speed(0.1));
            }
        });

        ui.separator();

        // 変数選択
        ui.heading("📊 Variables");

        if let Some(file_index) = self.selected_file_index {
            if let Some(data_file) = self.loaded_files.get(file_index) {
                ui.horizontal(|ui| {
                    if ui.button("All").clicked() {
                        self.plot_settings.visible_variables = data_file.variables.clone();
                    }
                    if ui.button("None").clicked() {
                        self.plot_settings.visible_variables.clear();
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for var_name in &data_file.variables {
                            let mut is_visible =
                                self.plot_settings.visible_variables.contains(var_name);
                            if ui.checkbox(&mut is_visible, var_name).changed() {
                                if is_visible {
                                    if !self.plot_settings.visible_variables.contains(var_name) {
                                        self.plot_settings.visible_variables.push(var_name.clone());
                                    }
                                } else {
                                    self.plot_settings
                                        .visible_variables
                                        .retain(|x| x != var_name);
                                }
                            }
                        }
                    });
            }
        } else {
            ui.label("Select a file to choose variables");
        }
    }

    fn data_table_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Data Table");
        ui.separator();

        if self.loaded_data.is_empty() {
            ui.label("No data to display");
            return;
        }

        // 表示行数設定
        ui.horizontal(|ui| {
            ui.label("Show last:");
            ui.add(
                egui::DragValue::new(&mut self.table_rows_to_show)
                    .range(10..=10000)
                    .suffix(" rows"),
            );
        });

        ui.separator();

        // フィルタされたデータを準備
        let filtered_data = self.get_filtered_data();
        let visible_vars: Vec<String> = self
            .plot_settings
            .visible_variables
            .iter()
            .filter(|var| filtered_data.contains_key(*var))
            .cloned()
            .collect();

        if visible_vars.is_empty() {
            ui.label("Select variables to display");
            return;
        }

        // 最大行数を決定
        let max_rows = visible_vars
            .iter()
            .filter_map(|var| filtered_data.get(var))
            .map(|data| data.len())
            .min()
            .unwrap_or(0)
            .min(self.table_rows_to_show);

        // テーブル表示
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .vscroll(true)
            .column(Column::initial(100.0).resizable(true)) // Timestamp列
            .columns(Column::initial(100.0).resizable(true), visible_vars.len())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Time (s)");
                });
                for var_name in &visible_vars {
                    header.col(|ui| {
                        ui.strong(var_name);
                    });
                }
            })
            .body(|mut body| {
                for i in 0..max_rows {
                    body.row(18.0, |mut row| {
                        // タイムスタンプ
                        row.col(|ui| {
                            if let Some(data) = filtered_data.get(&visible_vars[0]) {
                                if i < data.len() {
                                    ui.label(format!("{:.3}", data[i][0]));
                                } else {
                                    ui.label("--");
                                }
                            } else {
                                ui.label("--");
                            }
                        });

                        // 各変数の値
                        for var_name in &visible_vars {
                            row.col(|ui| {
                                if let Some(data) = filtered_data.get(var_name) {
                                    if i < data.len() {
                                        ui.label(format!("{:.6}", data[i][1]));
                                    } else {
                                        ui.label("--");
                                    }
                                } else {
                                    ui.label("--");
                                }
                            });
                        }
                    });
                }
            });
    }

    fn plot_display_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("📈 Data Plot");
        ui.separator();

        if self.loaded_data.is_empty() {
            ui.label("Load a data file to view plots");
            return;
        }

        if self.plot_settings.visible_variables.is_empty() {
            ui.label("Select variables to plot");
            return;
        }

        // フィルタされたデータを取得
        let filtered_data = self.get_filtered_data();

        if filtered_data.is_empty() {
            ui.label("No data available for plotting");
            return;
        }

        // プロット制御
        let mut reset_flag = false;
        ui.horizontal(|ui| {
            if ui.button("Pos Reset").clicked() {
                reset_flag = true;
            }
            ui.separator();
            ui.checkbox(&mut self.plot_settings.auto_range, "Auto Range");
            ui.checkbox(&mut self.plot_settings.show_points, "Show Points");

            ui.horizontal(|ui| {
                ui.label("Line Width:");
                ui.add(
                    egui::DragValue::new(&mut self.plot_settings.line_width)
                        .range(0.5..=5.0)
                        .speed(0.1),
                );
            });
        });

        ui.separator();

        // プロット作成
        let mut plot = Plot::new("data_viewer_plot")
            .legend(egui_plot::Legend::default())
            .allow_zoom(true)
            .allow_drag(true)
            .allow_scroll(true)
            .height(ui.available_height() - 200.0);

        if self.plot_settings.auto_range {
            plot = plot.auto_bounds([true, true].into());
        }

        if reset_flag {
            plot = plot.reset();
        }

        plot.show(ui, |plot_ui| {
            // 各変数のラインを描画
            for (i, var_name) in self.plot_settings.visible_variables.iter().enumerate() {
                if let Some(data) = filtered_data.get(var_name) {
                    if !data.is_empty() {
                        let points: PlotPoints =
                            data.iter().map(|[time, value]| [*time, *value]).collect();

                        let mut line = Line::new(points)
                            .name(var_name)
                            .width(self.plot_settings.line_width);

                        // 異なる色を設定
                        let colors = [
                            Color32::RED,
                            Color32::GREEN,
                            Color32::BLUE,
                            Color32::YELLOW,
                            Color32::LIGHT_BLUE,
                            Color32::LIGHT_GREEN,
                            Color32::LIGHT_RED,
                        ];
                        if let Some(color) = colors.get(i % colors.len()) {
                            line = line.color(*color);
                        }

                        plot_ui.line(line);

                        // ポイント表示
                        if self.plot_settings.show_points {
                            let points_for_display: PlotPoints =
                                data.iter().map(|[time, value]| [*time, *value]).collect();
                            plot_ui.points(
                                egui_plot::Points::new(points_for_display)
                                    .name(&format!("{}_points", var_name))
                                    .radius(2.0),
                            );
                        }
                    }
                }
            }
        });

        // 統計情報
        ui.separator();
        ui.collapsing("📊 Statistics", |ui| {
            for var_name in &self.plot_settings.visible_variables {
                if let Some(data) = filtered_data.get(var_name) {
                    if !data.is_empty() {
                        let values: Vec<f64> = data.iter().map(|[_, value]| *value).collect();
                        let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                        let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                        let avg_val = values.iter().sum::<f64>() / values.len() as f64;
                        let std_dev = {
                            let variance =
                                values.iter().map(|&x| (x - avg_val).powi(2)).sum::<f64>()
                                    / values.len() as f64;
                            variance.sqrt()
                        };

                        ui.label(format!(
                            "{}: Min={:.3}, Max={:.3}, Avg={:.3}, StdDev={:.3}, Samples={}",
                            var_name,
                            min_val,
                            max_val,
                            avg_val,
                            std_dev,
                            data.len()
                        ));
                    }
                }
            }
        });
    }

    fn get_filtered_data(&self) -> HashMap<String, Vec<[f64; 2]>> {
        let mut filtered = HashMap::new();

        for var_name in &self.plot_settings.visible_variables {
            if let Some(data) = self.loaded_data.get(var_name) {
                let filtered_data: Vec<[f64; 2]> = data
                    .iter()
                    .filter(|[time, _]| {
                        let after_start = self
                            .plot_settings
                            .time_range_start
                            .map_or(true, |start| *time >= start);
                        let before_end = self
                            .plot_settings
                            .time_range_end
                            .map_or(true, |end| *time <= end);
                        after_start && before_end
                    })
                    .cloned()
                    .collect();

                if !filtered_data.is_empty() {
                    filtered.insert(var_name.clone(), filtered_data);
                }
            }
        }

        filtered
    }

    fn load_csv_file(&mut self) {
        use rfd::FileDialog;

        if let Some(path) = FileDialog::new()
            .add_filter("CSV files", &["csv"])
            .pick_file()
        {
            self.loading_progress = 0.1;
            self.loading_message = "Loading CSV file...".to_string();
            self.last_error = None;

            match self.parse_csv_file(&path) {
                Ok(data_file) => {
                    self.loaded_files.push(data_file);
                    self.loading_progress = 1.0;
                    self.loading_message = "Load complete".to_string();

                    // 自動的に最新ファイルを選択
                    let new_index = self.loaded_files.len() - 1;
                    self.selected_file_index = Some(new_index);
                    self.load_file_data(new_index);
                }
                Err(e) => {
                    self.last_error = Some(e);
                    self.loading_progress = 0.0;
                }
            }
        }
    }

    fn parse_csv_file(&mut self, path: &PathBuf) -> Result<DataFile, String> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // ヘッダー行を読み込み
        let header_line = lines
            .next()
            .ok_or("Empty file")?
            .map_err(|e| format!("Failed to read header: {}", e))?;

        let headers: Vec<String> = header_line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        if headers.is_empty() || !headers[0].to_lowercase().contains("time") {
            return Err("Invalid CSV format: first column should be timestamp".to_string());
        }

        let variables = headers[1..].to_vec(); // Timestampを除く
        let mut sample_count = 0;
        let mut max_time: f64 = 0.0;

        // データ行をカウント（時間計算のため）
        for line_result in lines {
            match line_result {
                Ok(line) => {
                    if !line.trim().is_empty() {
                        sample_count += 1;

                        // 最後の時間を取得
                        if let Some(first_value) = line.split(',').next() {
                            if let Ok(time) = first_value.trim().parse::<f64>() {
                                max_time = max_time.max(time);
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(DataFile {
            path: path.clone(),
            name: file_name,
            variables,
            sample_count,
            duration: max_time,
        })
    }

    fn load_file_data(&mut self, file_index: usize) {
        let data_file_path = if let Some(data_file) = self.loaded_files.get(file_index) {
            data_file.path.clone()
        } else {
            return;
        };

        self.loading_progress = 0.2;
        self.loading_message = "Reading data...".to_string();

        match self.read_csv_data(&data_file_path) {
            Ok(data) => {
                self.loaded_data = data;
                self.loading_progress = 0.0; // 完了

                // 自動的に全変数を表示対象にする
                if self.plot_settings.visible_variables.is_empty() {
                    if let Some(data_file) = self.loaded_files.get(file_index) {
                        self.plot_settings.visible_variables = data_file.variables.clone();
                    }
                }
            }
            Err(e) => {
                self.last_error = Some(format!("Failed to load data: {}", e));
                self.loading_progress = 0.0;
            }
        }
    }

    fn read_csv_data(&mut self, path: &PathBuf) -> Result<HashMap<String, Vec<[f64; 2]>>, String> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // ヘッダー読み込み
        let header_line = lines
            .next()
            .ok_or("Empty file")?
            .map_err(|e| format!("Failed to read header: {}", e))?;

        let headers: Vec<String> = header_line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let variable_names = &headers[1..]; // Timestampを除く
        let mut data: HashMap<String, Vec<[f64; 2]>> = HashMap::new();

        // データ初期化
        for var_name in variable_names {
            data.insert(var_name.clone(), Vec::new());
        }

        // データ読み込み
        for line_result in lines {
            let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;

            if line.trim().is_empty() {
                continue;
            }

            let values: Vec<&str> = line.split(',').collect();

            if values.len() != headers.len() {
                continue; // 不正な行をスキップ
            }

            // タイムスタンプ解析
            let timestamp = values[0]
                .trim()
                .parse::<f64>()
                .map_err(|_| format!("Invalid timestamp: {}", values[0]))?;

            // 各変数の値を解析
            for (i, var_name) in variable_names.iter().enumerate() {
                if let Ok(value) = values[i + 1].trim().parse::<f64>() {
                    if let Some(var_data) = data.get_mut(var_name) {
                        var_data.push([timestamp, value]);
                    }
                }
            }
        }

        Ok(data)
    }

    fn clear_all_data(&mut self) {
        self.loaded_files.clear();
        self.selected_file_index = None;
        self.loaded_data.clear();
        self.plot_settings.visible_variables.clear();
        self.last_error = None;
        self.loading_progress = 0.0;
    }
}
