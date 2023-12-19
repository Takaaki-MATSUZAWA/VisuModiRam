use probe_rs::{MemoryInterface, Permissions, Probe, Session};

use std::sync::{Arc, Mutex};

use super::gdb_parser::VariableList;

pub struct ProbeInterface {
    target_mcu: String,
    watch_list: Arc<Mutex<Vec<VariableList>>>,
    watching_flag: Arc<Mutex<bool>>,
    temp_data: Arc<Mutex<i32>>,
}

impl Default for ProbeInterface {
    fn default() -> Self {
        Self {
            target_mcu: Default::default(),
            watch_list: Arc::new(Mutex::new(Vec::new())),
            watching_flag: Arc::new(Mutex::new(false)),
            temp_data: Arc::new(Mutex::new(0)),
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

    pub fn watching_start(
        &mut self,
        duration: std::time::Duration,
    ) -> std::thread::JoinHandle<Result<(), std::io::Error>> {
        let flag = self.watching_flag.clone();
        *flag.lock().unwrap() = true;

        let data = self.temp_data.clone();
        *data.lock().unwrap() = 0;

        if *flag.lock().unwrap() == false {
            return std::thread::spawn(move || {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "watching now",
                ))
            });
        }

        std::thread::spawn(move || {
            loop {
                if *flag.lock().unwrap() == false {
                    break;
                }

                let mut data_lock = data.lock().unwrap();
                *data_lock += 1;
                drop(data_lock);

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
}
