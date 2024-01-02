use super::gdb_parser::VariableInfo;
use probe_rs::{MemoryInterface, Permissions, Probe};
use sensorlog::{logfile_config::LogfileConfig, quota, time, Sensorlog};
use shellexpand;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Default, Clone, Debug)]
pub struct WatchSetting {
    pub target_mcu: String,
    pub probe_sn: String,
    pub watch_list: Vec<VariableInfo>,
}

#[derive(Clone)]
pub struct ProbeInterface {
    pub setting: WatchSetting,

    watching_flag: Arc<Mutex<bool>>,
    log_service: Arc<Mutex<Sensorlog>>,
    log_start_utime: u64,
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
            log_start_utime: 0,
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

        let _log_service = self.log_service.clone();

        self.log_start_utime = time::get_unix_microseconds().expect("get time error");

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
                if *flag.lock().unwrap() == false {
                    break;
                }

                for symbol in &setting.watch_list {
                    let val_name = symbol.name.clone();
                    let address = if symbol.address.starts_with("0x") {
                        u64::from_str_radix(&symbol.address[2..], 16)
                    } else {
                        symbol.address.parse::<u64>()
                    }
                    .expect("failed to parse watchlist variable address");
                    let val = core.read_word_32(address).map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                    })?;

                    let val = f32::from_bits(val);

                    _log_service
                        .lock()
                        .unwrap()
                        .store_measurement(
                            None,
                            &val_name.to_string(),
                            &format!("{:?}", val as f32),
                        )
                        .unwrap();
                }
                std::thread::sleep(duration);
            }
            Ok(())
        })
    }

    pub fn watching_stop(&mut self) {
        *self.watching_flag.lock().unwrap() = false;
    }

    pub fn get_newest_date(&mut self, index: String) -> Option<f64> {
        let now = time::get_unix_microseconds().expect("get time error");
        let time_ago = now - (1000000);
        let mut measurements = self
            .log_service
            .lock()
            .unwrap()
            .fetch_measurements(index.as_str(), None, Some(time_ago), None)
            .expect("log service load error");

        loop {
            let res = measurements.last();
            if let Some(val) = res {
                let res = val.data.parse::<f32>();
                measurements.pop();
                match res {
                    Ok(val) => return Some(val as f64),
                    Err(_) => continue,
                }
            } else {
                return None;
            }
        }
        /*
        let mut resval = None;
        let mut none_cnt = 0;
        loop {
            let res = measurements.last();
            if let Some(val) = res {
                let res = val.data.parse::<f32>();
                match res {
                    Ok(val) => resval = Some(val as f64),
                    Err(_) => resval = None,
                }
            } else {
                resval = None;
            }

            measurements.pop();
            if resval != None || none_cnt > 100 {
                break;
            }
            none_cnt += 1;
        }
        resval
         */
    }

    pub fn get_log_vec(&mut self, index: String) -> Vec<[f64; 2]> {
        let now = time::get_unix_microseconds().expect("get time error");
        let time_ago = now - (20000000);
        let measurements = self
            .log_service
            .lock()
            .unwrap()
            .fetch_measurements(index.as_str(), None, Some(time_ago), None)
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
