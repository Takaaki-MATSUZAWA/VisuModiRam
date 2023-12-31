use super::gdb_parser::VariableInfo;
use probe_rs::{MemoryInterface, Permissions, Probe, Session, Core};
use sensorlog::Sensorlog;
use shellexpand;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Default,Clone,Debug)]
pub struct WatchSetting {
    pub target_mcu: String,
    pub probe_sn: String,
    pub watch_list: Vec<VariableInfo>,
}

pub struct CoreHolder {
    core_session: Option<Arc<Mutex<Core<'static>>>>,
}

pub struct ProbeInterface2 {
    pub setting: WatchSetting,
    //session: Arc<Mutex<Option<Session>>>,
    //core_session: Arc<Mutex<Option<Core>>>,
    //core_holder: Arc<Mutex<CoreHolder>>,

    watch_list: Arc<Mutex<Vec<VariableInfo>>>,
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

impl Default for ProbeInterface2 {
    fn default() -> Self {
        Self {
            setting: Default::default(),
            watch_list: Arc::new(Mutex::new(Vec::new())),
            watching_flag: Arc::new(Mutex::new(false)),
            temp_data: Arc::new(Mutex::new(0)),
            log_service: Arc::new(Mutex::new(log_service_default())),
            log_start_utime: 0,
            //core_session: Arc::new(Mutex::new(None)),
            //session: Arc::new(Mutex::new(None)),
            //core_holder: Arc::new(Mutex::new(CoreHolder { core_session: None })),

        }
    }
}

impl ProbeInterface2 {
    pub fn set_probe(&mut self, setting: WatchSetting) -> Result<(), probe_rs::Error> {
        /*
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
        self.session = Arc::new(Mutex::new(Some(probe.attach(setting.target_mcu, Permissions::default())?)));
        let mut session_guard = self.session.lock().unwrap();
        let session = session_guard.as_mut().unwrap();
        let core_session = Box::leak(Box::new(session.core(0)?.into())) as &'static mut Core;
        let core_session = Arc::new(Mutex::new(core_session));
        let mut core_holder_guard = self.core_holder.lock().unwrap();
        core_holder_guard.core_session = Some(core_session);
        
        */
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

        let data = self.temp_data.clone();
        *data.lock().unwrap() = 0;

        let _log_service = self.log_service.clone();

        self.log_start_utime = sensorlog::time::get_unix_microseconds().expect("get time error");

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
            let mut session = probe.attach(setting.target_mcu.clone(), Permissions::default()).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            // Select a core.
            let mut core = session.core(0).or_else(|_| {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "No matching probe found"))
            })?;

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

                for symbol in &setting.watch_list{
                    let val_name = symbol.name.clone();
                    let address = if symbol.address.starts_with("0x") {
                        u64::from_str_radix(&symbol.address[2..], 16)
                    } else {
                        symbol.address.parse::<u64>()
                    }.expect("failed to parse watchlist variable address");
                    let val = core.read_word_32(address).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
                    
                    _log_service
                        .lock()
                        .unwrap()
                        .store_measurement(None, &val_name.to_string(), &format!("{}",val))
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

