use super::elf_parser::VariableInfo;
use probe_rs::{
    flashing::{self, DownloadOptions, FlashProgress, FormatKind},
    probe::{list::Lister, DebugProbeError},
    Permissions,
};
use sensorlog_ram::{logfile_config::SaveFormat, quota, LogfileConfig, Measurement, SensorlogRam};
use shellexpand;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use stopwatch::Stopwatch;
use tracing::{debug, error, info, warn};

use super::memory_interface::MCUMemory;
// ----------------------------------------------------------------------------
#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WatchSetting {
    pub target_mcu: String,
    pub probe_sn: String,
    pub watch_list: Vec<VariableInfo>,
}
// ----------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FlashProgressState {
    None,
    Erasing,
    Programing,
    Finished,

    Failed,
}

impl std::fmt::Display for FlashProgressState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Default for FlashProgressState {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Progress {
    pub state: FlashProgressState,
    pub progress: f64,
    pub total_bytes: u64,
    pub now_bytes: u64,
}

// ----------------------------------------------------------------------------
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ProbeInterface {
    pub setting: WatchSetting,
    #[cfg_attr(feature = "serde", serde(skip))]
    flash_progress: Arc<Mutex<Progress>>,

    #[cfg_attr(feature = "serde", serde(skip))]
    watching_flag: Arc<Mutex<bool>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    log_service: Arc<Mutex<SensorlogRam>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    write_que: Arc<Mutex<BTreeMap<VariableInfo, String>>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    log_timer: Arc<Mutex<Stopwatch>>,
}

fn log_service_default() -> SensorlogRam {
    let mut logfile_config = LogfileConfig::new();
    // RAMベースの高性能設定
    logfile_config.set_default_storage_quota(quota::StorageQuota::MaxMeasurements(100000)); // 100K測定値まで
    logfile_config.set_auto_save_interval(Some(30000)); // 30秒ごとに自動保存
    logfile_config.set_max_ram_usage(200 * 1024 * 1024); // 200MB RAM制限
    logfile_config.set_save_format(SaveFormat::Binary); // 高速バイナリ形式
    logfile_config.set_compression(false); // 圧縮なし（速度優先）

    let datadir = PathBuf::from(format!("{}", shellexpand::tilde("~/.EguiMonitorLog")));
    if !datadir.exists() {
        std::fs::create_dir_all(&datadir).expect("faild create logfile dir");
    }
    let service = SensorlogRam::new(&datadir, logfile_config).expect("faild create logfile");
    service
}

impl Default for ProbeInterface {
    fn default() -> Self {
        Self {
            setting: Default::default(),
            watching_flag: Arc::new(Mutex::new(false)),
            log_service: Arc::new(Mutex::new(log_service_default())),
            write_que: Arc::new(Mutex::new(BTreeMap::new())),
            log_timer: Arc::new(Mutex::new(Stopwatch::new())),
            flash_progress: Arc::new(Mutex::new(Default::default())),
        }
    }
}

impl ProbeInterface {
    pub fn set_probe(&mut self, setting: WatchSetting) -> Result<(), probe_rs::Error> {
        self.setting = setting.clone();
        Ok(())
    }

    pub fn watching_start(
        &mut self,
        duration: std::time::Duration,
    ) -> std::thread::JoinHandle<Result<(), std::io::Error>> {
        let flag = self.watching_flag.clone();
        if *flag.lock().unwrap() == true {
            return std::thread::spawn(move || Ok(()));
        }

        *flag.lock().unwrap() = true;

        //let _log_service = self.log_service.clone();
        let _log_service = Arc::clone(&self.log_service);
        let _write_que = Arc::clone(&self.write_que);

        let _log_timer = Arc::clone(&self.log_timer);
        _log_timer.lock().unwrap().start();

        let setting = self.setting.clone();

        std::thread::spawn(move || {
            let lister = Lister::new();
            let probes = lister.list_all();

            let probe_info = probes
                .into_iter()
                .find(|probe| probe.serial_number == Some(setting.probe_sn.clone()))
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "No matching probe found")
                })
                .unwrap();

            let probe = probe_info.open().unwrap();

            // Attach to a chip.
            let mut session = probe
                .attach(setting.target_mcu.clone(), Permissions::default())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            // Select a core.
            let mut core = session.core(0).or_else(|_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No matching probe found",
                ))
            })?;

            loop {
                let entry_time = std::time::Instant::now();
                if *flag.lock().unwrap() == false {
                    break;
                }

                for symbol in &setting.watch_list {
                    let val_str = MCUMemory::read(&mut core, symbol);
                    let now_time = _log_timer.lock().unwrap().elapsed_ms();

                    match _log_service.lock().unwrap().store_measurement(
                        Some(now_time as u64),
                        &symbol.name,
                        &val_str,
                    ) {
                        Ok(_) => {}
                        Err(_e) => {
                            #[cfg(debug_assertions)]
                            error!("測定値の保存中にエラーが発生しました: {}", _e);
                        }
                    }
                }
                let write_map = _write_que.lock().unwrap().clone();
                _write_que.lock().unwrap().clear();
                for que in write_map {
                    let _res = MCUMemory::write(&mut core, &que.0, &que.1);
                }

                let exit_time = std::time::Instant::now();

                let elapsed_time = exit_time.duration_since(entry_time);
                if let Some(sleep_duration) = duration.checked_sub(elapsed_time) {
                    std::thread::sleep(sleep_duration);
                }
            }
            Ok(())
        })
    }

    pub fn watching_stop(&mut self) {
        *self.watching_flag.lock().unwrap() = false;
        self.log_timer.lock().unwrap().stop();
    }

    pub fn now_watching(&mut self) -> bool {
        *self.watching_flag.lock().unwrap()
    }

    pub fn get_newest_date(&mut self, index: &str) -> Option<f64> {
        let now_time = self.log_timer.lock().unwrap().elapsed_ms();
        let last_time = now_time - 500;

        // 時間範囲を修正: time_start（過去）からtime_end（現在）
        let measurements =
            self.load_data(index, Some(last_time as u64), Some(now_time as u64), None);

        let res = measurements.last();
        if let Some(val) = res {
            let res = val.data.parse::<f32>();
            match res {
                Ok(val) => return Some(val as f64),
                Err(_) => return None,
            }
        } else {
            return None;
        }
    }

    // If time_window is None, data for the entire period is returned after starting measurement.
    pub fn get_log_vec(&mut self, index: &str, time_window: Option<u64>) -> Vec<[f64; 2]> {
        let now_time = self.log_timer.lock().unwrap().elapsed_ms();
        let last_time = if time_window == None {
            None
        } else {
            let mut last_time = now_time - (time_window.unwrap() as i64);
            if last_time < 0 {
                last_time = 0;
            }
            Some(last_time as u64)
        };

        // 時間範囲を修正: time_start（過去）からtime_end（現在）
        let measurements = self.load_data(index, last_time, Some(now_time as u64), None);

        let mut vec = Vec::new();
        for measurement in measurements {
            let time = (measurement.time) as f64 / 1000.0;
            let data = measurement.data.parse::<f64>().unwrap();
            vec.push([time, data]);
        }
        vec
    }

    fn load_data(
        &mut self,
        index: &str,
        time_start: Option<u64>,
        time_end: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<Measurement> {
        self.log_service
            .lock()
            .unwrap()
            .fetch_measurements(index, time_start, time_end, limit)
            .expect("log service load error")
    }

    pub fn insert_wirte_que(&mut self, symbol: &VariableInfo, data: &str) {
        self.write_que
            .lock()
            .unwrap()
            .insert(symbol.clone(), data.to_string());
    }

    // sensorlog-ram特有の機能
    pub fn get_memory_usage(&mut self) -> usize {
        self.log_service.lock().unwrap().get_memory_usage()
    }

    pub fn get_sensor_names(&mut self) -> Vec<String> {
        self.log_service.lock().unwrap().get_sensor_names()
    }

    pub fn get_measurement_count(&mut self, sensor_name: &str) -> usize {
        self.log_service
            .lock()
            .unwrap()
            .get_measurement_count(sensor_name)
    }

    pub fn force_save_to_disk(&mut self) -> Result<(), std::io::Error> {
        self.log_service
            .lock()
            .unwrap()
            .save_to_disk()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }

    pub fn clear_sensor_ram(&mut self, sensor_name: &str) {
        self.log_service
            .lock()
            .unwrap()
            .clear_sensor_ram(sensor_name);
    }

    pub fn get_flash_progress(&mut self) -> Progress {
        self.flash_progress.lock().unwrap().clone()
    }

    pub fn flash(
        &mut self,
        elf_path: PathBuf,
    ) -> std::thread::JoinHandle<Result<(), probe_rs::Error>> {
        let lister = Lister::new();
        let probes = lister.list_all();
        let setting = self.setting.clone();

        if self.now_watching() {
            error!("Cannot flash while watching is active");
            let err = Err(probe_rs::Error::Probe(DebugProbeError::Attached));
            return std::thread::spawn(move || err);
        }

        info!("Starting firmware flash to {:?}", elf_path);
        self.flash_progress.lock().unwrap().state = FlashProgressState::Erasing;
        let progress_clone = Arc::clone(&self.flash_progress);
        let progress_clone2 = Arc::clone(&self.flash_progress);

        std::thread::spawn(move || {
            info!("Flash thread started");

            let probe_info = probes
                .into_iter()
                .find(|probe| probe.serial_number == Some(setting.probe_sn.clone()))
                .ok_or_else(|| {
                    error!("No matching probe found with SN: {}", setting.probe_sn);
                    probe_rs::Error::Other("No matching probe found".to_string())
                })?;

            info!("Found matching probe: {:?}", probe_info.serial_number);
            let probe = probe_info.open().map_err(|e| {
                error!("Failed to open probe: {:?}", e);
                probe_rs::Error::Probe(e)
            })?;

            info!(
                "Probe opened successfully, attaching to target: {}",
                setting.target_mcu
            );
            let mut session = probe
                .attach(setting.target_mcu.clone(), Permissions::default())
                .map_err(|e| {
                    error!("Failed to attach to target {}: {:?}", setting.target_mcu, e);
                    e
                })?;

            info!("Successfully attached to target");

            // Set up progress tracking with probe-rs 0.29.0 API (simplified)
            let progress = FlashProgress::new(move |event| {
                debug!("Flash progress event: {:?}", event);
                let mut progress = progress_clone.lock().unwrap();
                // Note: probe-rs 0.29.0 ProgressEvent structure changed
                // For now, we'll just track basic progress without detailed events
                progress.state = FlashProgressState::Programing;
            });

            let mut options = DownloadOptions::default();
            options.progress = Some(progress);

            info!("Starting firmware download");
            let flash_result = probe_rs::flashing::download_file_with_options(
                &mut session,
                elf_path.clone(),
                FormatKind::Elf,
                options,
            );

            match flash_result {
                Ok(_) => {
                    info!("Firmware flash completed successfully");
                    progress_clone2.lock().unwrap().state = FlashProgressState::Finished;
                }
                Err(e) => {
                    error!("Firmware flash failed: {:?}", e);
                    progress_clone2.lock().unwrap().state = FlashProgressState::Failed;
                    return Err(probe_rs::Error::Other(format!("Flash failed: {}", e)));
                }
            }

            // Reset target
            info!("Resetting target after flash");
            match session.core(0).and_then(|mut core| {
                info!("Resetting target...");
                core.reset()?;
                info!("Starting target execution...");
                core.run()?;
                Ok(())
            }) {
                Ok(_) => {
                    info!("Target reset and start successful");
                }
                Err(e) => {
                    warn!(
                        "Target reset failed: {:?} (firmware flash was successful)",
                        e
                    );
                    // Don't fail the entire operation for reset issues
                }
            }

            info!("Flash operation completed");
            Ok(())
        })
    }
}

// ----------------------------------------------------------------------------
