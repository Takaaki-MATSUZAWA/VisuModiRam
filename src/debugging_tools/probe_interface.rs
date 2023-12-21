use super::gdb_parser::VariableList;
use probe_rs::{MemoryInterface, Permissions, Probe, Session};
use sensorlog::Sensorlog;
use shellexpand;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ProbeInterface {
    target_mcu: String,
    watch_list: Arc<Mutex<Vec<VariableList>>>,
    watching_flag: Arc<Mutex<bool>>,
    temp_data: Arc<Mutex<i32>>,
    log_service: Arc<Mutex<sensorlog::Sensorlog>>,
    log_start_utime: u64,
}

fn log_service_default() -> sensorlog::Sensorlog {
    let mut logfile_config = sensorlog::logfile_config::LogfileConfig::new();
    logfile_config.set_default_storage_quota(sensorlog::quota::StorageQuota::Unlimited);

    let datadir = PathBuf::from(format!("{}", shellexpand::tilde("~/.EguiMonitorLog")));
    if !datadir.exists() {
        std::fs::create_dir_all(&datadir).expect("faild create logfile dir");
    }
    let service =
        sensorlog::Sensorlog::new(&datadir, logfile_config).expect("faild create logfile");
    service
}

impl Default for ProbeInterface {
    fn default() -> Self {
        Self {
            target_mcu: Default::default(),
            watch_list: Arc::new(Mutex::new(Vec::new())),
            watching_flag: Arc::new(Mutex::new(false)),
            temp_data: Arc::new(Mutex::new(0)),
            log_service: Arc::new(Mutex::new(log_service_default())),
            log_start_utime: 0,
        }
    }
}

impl ProbeInterface {
    pub fn get_connected_probes(&mut self) -> Vec<probe_rs::DebugProbeInfo> {
        Probe::list_all()
    }

    pub fn set_probe(
        &mut self,
        mcu: String,
        probe: probe_rs::DebugProbeInfo,
    ) -> Result<(), std::io::Error> {
        // TODO : mcuがprobe_rsのターゲットに一致するか確認
        self.target_mcu = mcu;

        //self.selected_probe = probe;
        //let duration = std::time::Duration::from_millis(1000); // 1000ミリ秒、つまり1秒

        Ok(())
    }

    pub fn prepare_logging() {}

    pub fn watching_start(
        &mut self,
        duration: std::time::Duration,
    ) -> std::thread::JoinHandle<Result<(), std::io::Error>> {
        let flag = self.watching_flag.clone();
        if *flag.lock().unwrap() == true {
            return std::thread::spawn(move || Ok(()));
        }

        *flag.lock().unwrap() = true;

        let data = self.temp_data.clone();
        *data.lock().unwrap() = 0;

        let _log_service = self.log_service.clone();

        self.log_start_utime = sensorlog::time::get_unix_microseconds().expect("get time error");

        std::thread::spawn(move || {
            loop {
                if *flag.lock().unwrap() == false {
                    break;
                }

                *data.lock().unwrap() += 1;
                _log_service
                    .lock()
                    .unwrap()
                    .store_measurement(None, "temp_data", &(*data.lock().unwrap()).to_string())
                    .unwrap();
                std::thread::sleep(duration);
            }
            Ok(())
        })
    }

    pub fn watching_stop(&mut self) {
        *self.watching_flag.lock().unwrap() = false;
    }

    pub fn get_data(&mut self) -> i32 {
        *self.temp_data.lock().unwrap()
    }

    pub fn get_log_vec(&mut self) -> Vec<[f64; 2]> {
        let now = sensorlog::time::get_unix_microseconds().expect("get time error");
        let time_ago = now - (20000000);
        let measurements = self
            .log_service
            .lock()
            .unwrap()
            .fetch_measurements("temp_data", None, Some(time_ago), None)
            .expect("log service load error");

        let mut vec = Vec::new();
        for measurement in measurements {
            if measurement.time < self.log_start_utime {
                continue;
            }
            let time = (measurement.time - self.log_start_utime) as f64 / 1000000.0;
            let data = measurement.data.parse::<f64>().unwrap();
            vec.push([time, data]);
        }
        vec
    }
}
