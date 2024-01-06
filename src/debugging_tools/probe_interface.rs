use super::gdb_parser::VariableInfo;
use probe_rs::{MemoryInterface, Permissions, Probe, Core};
use sensorlog::{logfile_config::LogfileConfig, quota, time, Sensorlog, measure::Measurement};
use shellexpand;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;

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
    #[cfg_attr(feature = "serde", serde(skip))]
    write_que: Arc<Mutex<BTreeMap<VariableInfo, String>>>,
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
            write_que: Arc::new(Mutex::new(BTreeMap::new())),
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
                let write_map = _write_que.lock().unwrap().clone();
                for que in write_map {
                    let _res = Self::write_mem(&mut core, &que.0, &que.1);
                }
                _write_que.lock().unwrap().clear();
                std::thread::sleep(duration);
            }
            Ok(())
        })
    }

    fn write_mem(core: &mut Core, symbol: &VariableInfo, value_str: &str) -> Result<(),probe_rs::Error>{
        let c_type = symbol.types.as_str();
        let c_type = c_type.strip_prefix("volatile ").unwrap_or(&c_type);
        let c_type = c_type.strip_suffix(" [").unwrap_or(c_type);
        match c_type {
            "signed char" | "char"  => core.write_word_8(symbol.address, value_str.parse::<f64>().unwrap() as i8 as u8),
            "unsigned char"         => core.write_word_8(symbol.address, value_str.parse::<f64>().unwrap() as u8),
            "short"                 => {
                let buf = value_str.parse::<f64>().unwrap() as i16;
                let block = buf.to_le_bytes();
                core.write_8(symbol.address, &block).map_err(|e| e.into())
            },
            "unsigned short"        => {
                let buf = value_str.parse::<f64>().unwrap() as u16;
                let block = buf.to_le_bytes();
                core.write_8(symbol.address, &block).map_err(|e| e.into())
            },
            "int" | "long"          => core.write_word_32(symbol.address, value_str.parse::<f64>().unwrap() as i32 as u32),
            "unsigned int"|"unsigned long"=> core.write_word_32(symbol.address, value_str.parse::<f64>().unwrap() as u32 as u32),
            "long long"             => {
                let buf = value_str.parse::<f64>().unwrap() as i64;
                let block = buf.to_le_bytes();
                let block_u32 = [
                    u32::from_le_bytes([block[0], block[1], block[2], block[3]]),
                    u32::from_le_bytes([block[4], block[5], block[6], block[7]])
                ];
                core.write_32(symbol.address, &block_u32).map_err(|e| e.into())
            },
            "unsigned long long"    => {
                let buf = value_str.parse::<f64>().unwrap() as u64;
                let block = buf.to_le_bytes();
                let block_u32 = [
                    u32::from_le_bytes([block[0], block[1], block[2], block[3]]),
                    u32::from_le_bytes([block[4], block[5], block[6], block[7]])
                ];
                core.write_32(symbol.address, &block_u32).map_err(|e| e.into())
            },
            "float"                 => core.write_word_32(symbol.address, value_str.parse::<f32>().unwrap().to_bits()),
            "double"|"long double"  => core.write_word_64(symbol.address, value_str.parse::<f64>().unwrap().to_bits()),
            _ => Err(probe_rs::Error::Other(anyhow::anyhow!("Unsupported type"))),
        }
    }

    fn read_mem(core: &mut Core, symbol: &VariableInfo) -> String{
        let c_type = symbol.types.as_str();
        let c_type = c_type.strip_prefix("volatile ").unwrap_or(&c_type);
        let c_type = c_type.strip_suffix(" [").unwrap_or(c_type);
        let val_str = {
            match c_type {
                "char" | "signed char"  => {
                    let val_bits = core.read_word_8(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    format!("{}",val_bits as i8)
                },
                "unsigned char"         => {
                    let val_bits = core.read_word_8(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    format!("{}",val_bits)
                },
                "short"                 => {
                    let mut buff = [0u8; 2];
                    core.read_8(symbol.address, &mut buff).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    let val_bits = u16::from_le_bytes(buff);
                    format!("{}",val_bits as i16)
                },
                "unsigned short"        => {
                    let mut buff = [0u8; 2];
                    core.read_8(symbol.address, &mut buff).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    let val_bits = u16::from_le_bytes(buff);
                    format!("{}",val_bits)
                },
                "int" | "long"          => {
                    let val_bits = core.read_word_32(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    format!("{}",val_bits as i32)
                },
                "unsigned int"|"unsigned long" => {
                    let val_bits = core.read_word_32(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    format!("{}",val_bits)
                },
                "long long"             => {
                    let val_bits = core.read_word_64(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    format!("{}",val_bits as i64)
                },
                "unsigned long long"    => {
                    let val_bits = core.read_word_64(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    format!("{}",val_bits)
                },
                "float"                 => {
                    let val_bits = core.read_word_32(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    format!("{:?}",f32::from_bits(val_bits))

                },
                "double" |"long double"=> {
                    // long double cast to double
                    let val_bits = core.read_word_64(symbol.address).map_err(|e| {std::io::Error::new(std::io::ErrorKind::Other, e.to_string())}).unwrap();
                    format!("{:?}",f64::from_bits(val_bits))
                },
                _ => format!(""),
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

    pub fn insert_wirte_que(&mut self, symbol: &VariableInfo, data: &str){
        self.write_que
            .lock()
            .unwrap()
            .insert(symbol.clone(), data.to_string());
    }
}

// ----------------------------------------------------------------------------

