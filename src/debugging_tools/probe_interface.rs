use super::elf_parser::VariableInfo;
use probe_rs::{
    flashing::{self, DownloadOptions, FlashProgress},
    DebugProbeError, Permissions, Probe,
};
use sensorlog::{logfile_config::LogfileConfig, measure::Measurement, quota, Sensorlog};
use shellexpand;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use stopwatch::Stopwatch;

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
    log_service: Arc<Mutex<Sensorlog>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    write_que: Arc<Mutex<BTreeMap<VariableInfo, String>>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    log_timer: Arc<Mutex<Stopwatch>>,
}

fn log_service_default() -> Sensorlog {
    let mut logfile_config = LogfileConfig::new();
    logfile_config.set_default_storage_quota(quota::StorageQuota::Unlimited);

    let datadir = PathBuf::from(format!("{}", shellexpand::tilde("~/.EguiMonitorLog")));
    if !datadir.exists() {
        std::fs::create_dir_all(&datadir).expect("faild create logfile dir");
    }
    let service = Sensorlog::new(&datadir, logfile_config).expect("faild create logfile");
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
            let probes = Probe::list_all();

            let probe = probes
                .into_iter()
                .find(|probe| probe.serial_number == Some(setting.probe_sn.clone()))
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "No matching probe found")
                })
                .unwrap()
                .open()
                .unwrap();

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
                            println!("測定値の保存中にエラーが発生しました: {}", _e);
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

        let measurements =
            self.load_data(index, Some(now_time as u64), Some(last_time as u64), None);

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

        let measurements = self.load_data(index, Some(now_time as u64), last_time, None);

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
        time_limit: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<Measurement> {
        self.log_service
            .lock()
            .unwrap()
            .fetch_measurements(index, time_start, time_limit, limit)
            .expect("log service load error")
    }

    pub fn insert_wirte_que(&mut self, symbol: &VariableInfo, data: &str) {
        self.write_que
            .lock()
            .unwrap()
            .insert(symbol.clone(), data.to_string());
    }

    pub fn get_flash_progress(&mut self) -> Progress {
        self.flash_progress.lock().unwrap().clone()
    }

    //pub fn flash(&mut self, elf_path: PathBuf) -> Result<(), probe_rs::Error> {
    pub fn flash(
        &mut self,
        elf_path: PathBuf,
    ) -> std::thread::JoinHandle<Result<(), probe_rs::Error>> {
        use flashing::ProgressEvent::*;

        let probes = Probe::list_all();
        let setting = self.setting.clone();

        if self.now_watching() {
            let err = Err(probe_rs::Error::Probe(DebugProbeError::Attached));
            return std::thread::spawn(move || err);
        }

        self.flash_progress.lock().unwrap().state = FlashProgressState::Erasing;
        let progress_clone = Arc::clone(&self.flash_progress);

        std::thread::spawn(move || {
            let probe = probes
                .into_iter()
                .find(|probe| probe.serial_number == Some(setting.probe_sn.clone()))
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "No matching probe found")
                })
                .unwrap()
                .open()
                .unwrap();

            // Attach to a chip.
            let mut session = probe.attach(setting.target_mcu.clone(), Permissions::default())?;

            let total_page_size = Arc::new(Mutex::new(0u32));
            let total_sector_size = Arc::new(Mutex::new(0u64));

            // Register callback to update the progress.
            let progress = FlashProgress::new(move |event| {
                let mut progress = progress_clone.lock().unwrap();
                let mut total_page_size = total_page_size.lock().unwrap();
                let mut total_sector_size = total_sector_size.lock().unwrap();

                match event {
                    Initialized { flash_layout } => {
                        *total_page_size = flash_layout.pages().iter().map(|s| s.size()).sum();
                        *total_sector_size = flash_layout.sectors().iter().map(|s| s.size()).sum();
                    }
                    StartedProgramming => {
                        progress.state = FlashProgressState::Programing;
                        progress.progress = 0.0;
                    }
                    StartedErasing => {
                        progress.state = FlashProgressState::Erasing;
                        progress.progress = 0.0;
                    }
                    StartedFilling => {}
                    PageProgrammed { size, .. } => {
                        progress.progress += (size as f64) / (total_page_size.clone() as f64);
                    }
                    SectorErased { size, .. } => {
                        progress.progress += (size as f64) / (total_sector_size.clone() as f64);
                    }
                    PageFilled { .. } => {}
                    FailedErasing => {
                        progress.state = FlashProgressState::Failed;
                    }
                    FinishedErasing => {
                        progress.state = FlashProgressState::Failed;
                    }
                    FailedProgramming => {
                        progress.state = FlashProgressState::Failed;
                    }
                    FinishedProgramming => {
                        progress.state = FlashProgressState::Finished;
                    }
                    FailedFilling => {
                        progress.state = FlashProgressState::Failed;
                    }
                    FinishedFilling => {}
                    DiagnosticMessage { .. } => todo!(),
                }
            });

            let mut options = DownloadOptions::default();
            options.progress = Some(progress);

            let _res = flashing::download_file_with_options(
                &mut session,
                elf_path,
                flashing::Format::Elf,
                options,
            );

            #[cfg(debug_assertions)]
            println!("flash {:?}", _res);

            // Reset target according to CLI options
            {
                let mut core = session.core(0)?;

                core.reset()?;
            }
            Ok(())
        })
    }
}

// ----------------------------------------------------------------------------
