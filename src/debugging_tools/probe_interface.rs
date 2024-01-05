use super::gdb_parser::VariableInfo;
use probe_rs::{MemoryInterface, Permissions, Probe, Core};
use sensorlog::{logfile_config::LogfileConfig, quota, time, Sensorlog, measure::Measurement};
use shellexpand;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WatchSetting {
    pub target_mcu: String,
    pub probe_sn: String,
    pub watch_list: Vec<VariableInfo>,
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ProbeInterface {
    pub setting: WatchSetting,

    #[cfg_attr(feature = "serde", serde(skip))]
    watching_flag: Arc<Mutex<bool>>,
    #[cfg_attr(feature = "serde", serde(skip))]
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

        //let _log_service = self.log_service.clone();
        let _log_service = Arc::clone(&self.log_service);

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
                    let val_str = Self::read_mem(&mut core ,symbol);

                    match _log_service.lock().unwrap().store_measurement(None, &symbol.name, &val_str) {
                        Ok(_) => {},
                        Err(e) => {
                            // ここでエラーを処理します。例えば、それをログに記録します
                            println!("測定値の保存中にエラーが発生しました: {}", e);
                        },
                    }
                }
                std::thread::sleep(duration);
            }
            Ok(())
        })
    }

    fn read_mem(core: &mut Core, symbol: &VariableInfo) -> String{
        let bit_width = get_bit_width(&symbol.types);
                    
        let val_str = {
            match bit_width {
                Some(Symbolbitwidth::BW8)  => {
                    let val_bits = core.read_word_8(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    if is_unsigned(&symbol.types){
                        format!("{}",val_bits)
                    }else{
                        format!("{}",val_bits as i8)
                    }
                },
                Some(Symbolbitwidth::BW16)  => {
                    let mut buff = [0u8; 2];
                    core.read_8(symbol.address, &mut buff).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    let val_bits = u16::from_le_bytes(buff);
                    if is_unsigned(&symbol.types){
                        format!("{}",val_bits)
                    }else{
                        format!("{}",val_bits as i16)
                    }
                },
                Some(Symbolbitwidth::BW32)  => {
                    let val_bits = core.read_word_32(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    if is_unsigned(&symbol.types){
                        format!("{}",val_bits)
                    }else{
                        if symbol.types.contains("float") {
                            format!("{:?}",f32::from_bits(val_bits))
                        }else{
                            format!("{}",val_bits as i32)
                        }
                    }
                },
                Some(Symbolbitwidth::BW64)  => {
                    let val_bits = core.read_word_64(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    if is_unsigned(&symbol.types){
                        format!("{}",val_bits)
                    }else{
                        if symbol.types.contains("double") {
                            format!("{:?}",f64::from_bits(val_bits))
                        }else{
                            format!("{}",val_bits as i64)
                        }
                    }
                },
                _ => {
                    let val_bits = core.read_word_32(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    if is_unsigned(&symbol.types){
                        format!("{}",val_bits)
                    }else{
                        if symbol.types.contains("float") {
                            format!("{:?}",f32::from_bits(val_bits))
                        }else{
                            format!("{}",val_bits as i32)
                        }
                    }
                },
            }
        };
        val_str
    }

    pub fn watching_stop(&mut self) {
        *self.watching_flag.lock().unwrap() = false;
    }

    pub fn get_newest_date(&mut self, index: &str) -> Option<f64> {
        let now = time::get_unix_microseconds().expect("get time error");
        let time_ago = now - (1000000);
        let mut measurements = self.load_data(index, None, Some(time_ago), None);

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
    }

    pub fn get_log_vec(&mut self, index: &str) -> Vec<[f64; 2]> {
        let now = time::get_unix_microseconds().expect("get time error");
        let time_ago = now - (20000000);
        let measurements = self.load_data(index, None, Some(time_ago), None);

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

    fn load_data(&mut self, index: &str, time_start: Option<u64>, time_limit: Option<u64>, limit: Option<u64>) -> Vec<Measurement>{
        self.log_service
            .lock()
            .unwrap()
            .fetch_measurements(index, time_start, time_limit, limit)
            .expect("log service load error")
    }
}

// ----------------------------------------------------------------------------
enum Symbolbitwidth {
    BW8,
    BW16,
    BW32,
    BW64,
    BW128,
}

fn get_bit_width(c_type: &str) -> Option<Symbolbitwidth> {
    let c_type = String::from(c_type);
    let c_type = c_type.strip_prefix("volatile ").unwrap_or(&c_type);
    let c_type = c_type.strip_suffix(" [").unwrap_or(c_type);
    match c_type {
        "char"                  => Some(Symbolbitwidth::BW8),
        "unsigned char"         => Some(Symbolbitwidth::BW8),
        "signed char"           => Some(Symbolbitwidth::BW8),
        "short"                 => Some(Symbolbitwidth::BW16),
        "unsigned short"        => Some(Symbolbitwidth::BW16),
        "int"                   => Some(Symbolbitwidth::BW32),
        "unsigned int"          => Some(Symbolbitwidth::BW32),
        "long"                  => Some(Symbolbitwidth::BW32),
        "unsigned long"         => Some(Symbolbitwidth::BW32),
        "long long"             => Some(Symbolbitwidth::BW64),
        "unsigned long long"    => Some(Symbolbitwidth::BW64),
        "float"                 => Some(Symbolbitwidth::BW32),
        "double"                => Some(Symbolbitwidth::BW64),
        "long double"           => Some(Symbolbitwidth::BW128),
        _ => None,
    }
}

fn is_unsigned(c_type: &str) -> bool {
    c_type.contains("unsigned")
}
