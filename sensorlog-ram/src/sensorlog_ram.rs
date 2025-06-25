use crate::{
    logfile_config::{LogfileConfig, SaveFormat},
    measurement::Measurement,
    quota::StorageQuota,
    Result,
};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// RAM-based sensor logging system with periodic disk persistence
pub struct SensorlogRam {
    /// Configuration
    config: LogfileConfig,
    /// Base directory for saving files
    base_dir: PathBuf,
    /// In-memory storage: sensor_name -> measurements (sorted by timestamp)
    data: Arc<Mutex<HashMap<String, BTreeMap<u64, Measurement>>>>,
    /// Storage quotas per sensor
    quotas: Arc<Mutex<HashMap<String, StorageQuota>>>,
    /// Last save timestamp
    last_save_time: Arc<Mutex<Instant>>,
    /// Current memory usage estimate
    memory_usage: Arc<Mutex<usize>>,
    /// Flag to stop background save task
    should_stop: Arc<Mutex<bool>>,
}

impl SensorlogRam {
    /// Create a new SensorlogRam instance
    pub fn new<P: AsRef<Path>>(base_dir: P, config: LogfileConfig) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        
        // Create directory if it doesn't exist
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir)?;
        }

        let instance = Self {
            config,
            base_dir,
            data: Arc::new(Mutex::new(HashMap::new())),
            quotas: Arc::new(Mutex::new(HashMap::new())),
            last_save_time: Arc::new(Mutex::new(Instant::now())),
            memory_usage: Arc::new(Mutex::new(0)),
            should_stop: Arc::new(Mutex::new(false)),
        };

        // Start background save task if auto-save is enabled
        if let Some(interval_ms) = instance.config.auto_save_interval_ms {
            instance.start_auto_save_task(interval_ms);
        }

        Ok(instance)
    }

    /// Store a measurement for a given sensor
    pub fn store_measurement(
        &self,
        timestamp: Option<u64>,
        sensor_name: &str,
        data: &str,
    ) -> Result<()> {
        let timestamp = timestamp.unwrap_or_else(|| crate::current_timestamp_ms());
        let measurement = Measurement::with_timestamp(timestamp, data.to_string());

        let mut data_lock = self.data.lock().unwrap();
        let mut quotas_lock = self.quotas.lock().unwrap();
        let mut memory_lock = self.memory_usage.lock().unwrap();

        // Get or create sensor data
        let sensor_data = data_lock.entry(sensor_name.to_string()).or_insert_with(BTreeMap::new);
        
        // Get quota for this sensor
        let quota = quotas_lock
            .entry(sensor_name.to_string())
            .or_insert_with(|| self.config.default_storage_quota.clone());

        // Check if we need to remove old measurements
        let current_count = sensor_data.len();
        let oldest_time = sensor_data.keys().next().copied();
        let remove_count = quota.measurements_to_remove(current_count, *memory_lock, oldest_time, timestamp);
        
        // Remove old measurements if needed
        if remove_count > 0 {
            let keys_to_remove: Vec<_> = sensor_data.keys().take(remove_count).copied().collect();
            for key in keys_to_remove {
                if let Some(removed) = sensor_data.remove(&key) {
                    *memory_lock = memory_lock.saturating_sub(estimate_measurement_size(&removed));
                }
            }
        }

        // Add new measurement
        let measurement_size = estimate_measurement_size(&measurement);
        sensor_data.insert(timestamp, measurement);
        *memory_lock += measurement_size;

        // Check if we need to force a save due to memory usage
        if *memory_lock > self.config.max_ram_usage_bytes {
            drop(data_lock);
            drop(quotas_lock);
            drop(memory_lock);
            self.force_save()?;
        }

        Ok(())
    }

    /// Fetch measurements for a sensor within a time range
    pub fn fetch_measurements(
        &self,
        sensor_name: &str,
        time_start: Option<u64>,
        time_end: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<Measurement>> {
        let data_lock = self.data.lock().unwrap();
        
        let sensor_data = match data_lock.get(sensor_name) {
            Some(data) => data,
            None => return Ok(Vec::new()),
        };

        let mut measurements = Vec::new();
        let mut count = 0;

        for (&timestamp, measurement) in sensor_data.iter() {
            // Check time range
            if let Some(start) = time_start {
                if timestamp < start {
                    continue;
                }
            }
            if let Some(end) = time_end {
                if timestamp > end {
                    break;
                }
            }

            measurements.push(measurement.clone());
            count += 1;

            // Check limit
            if let Some(max_count) = limit {
                if count >= max_count {
                    break;
                }
            }
        }

        Ok(measurements)
    }

    /// Get list of all sensor names
    pub fn get_sensor_names(&self) -> Vec<String> {
        let data_lock = self.data.lock().unwrap();
        data_lock.keys().cloned().collect()
    }

    /// Set storage quota for a specific sensor
    pub fn set_sensor_quota(&self, sensor_name: &str, quota: StorageQuota) {
        let mut quotas_lock = self.quotas.lock().unwrap();
        quotas_lock.insert(sensor_name.to_string(), quota);
    }

    /// Get current memory usage in bytes (estimate)
    pub fn get_memory_usage(&self) -> usize {
        *self.memory_usage.lock().unwrap()
    }

    /// Get number of measurements for a sensor
    pub fn get_measurement_count(&self, sensor_name: &str) -> usize {
        let data_lock = self.data.lock().unwrap();
        data_lock.get(sensor_name).map(|data| data.len()).unwrap_or(0)
    }

    /// Save all data to disk immediately
    pub fn save_to_disk(&self) -> Result<()> {
        let data_lock = self.data.lock().unwrap();
        
        for (sensor_name, measurements) in data_lock.iter() {
            let file_path = self.get_sensor_file_path(sensor_name);
            self.save_sensor_data(sensor_name, measurements, &file_path)?;
        }

        *self.last_save_time.lock().unwrap() = Instant::now();
        Ok(())
    }

    /// Load data from disk for a specific sensor
    pub fn load_from_disk(&self, sensor_name: &str) -> Result<Vec<Measurement>> {
        let file_path = self.get_sensor_file_path(sensor_name);
        
        if !file_path.exists() {
            return Ok(Vec::new());
        }

        match self.config.save_format {
            SaveFormat::Binary => self.load_binary_data(&file_path),
            SaveFormat::Json => self.load_json_data(&file_path),
            SaveFormat::Csv => self.load_csv_data(&file_path),
        }
    }

    /// Clear all data from RAM (does not affect saved data)
    pub fn clear_ram(&self) {
        let mut data_lock = self.data.lock().unwrap();
        let mut memory_lock = self.memory_usage.lock().unwrap();
        
        data_lock.clear();
        *memory_lock = 0;
    }

    /// Clear data for a specific sensor from RAM
    pub fn clear_sensor_ram(&self, sensor_name: &str) {
        let mut data_lock = self.data.lock().unwrap();
        let mut memory_lock = self.memory_usage.lock().unwrap();
        
        if let Some(sensor_data) = data_lock.remove(sensor_name) {
            for measurement in sensor_data.values() {
                *memory_lock = memory_lock.saturating_sub(estimate_measurement_size(measurement));
            }
        }
    }

    /// Force a save if memory usage is high
    fn force_save(&self) -> Result<()> {
        self.save_to_disk()?;
        
        // Optionally clear RAM after save to free memory
        // This is a design decision - you might want to keep recent data in RAM
        let current_usage = self.get_memory_usage();
        if current_usage > self.config.max_ram_usage_bytes / 2 {
            // Keep only the most recent data in RAM
            self.trim_ram_data();
        }
        
        Ok(())
    }

    /// Start background auto-save task
    fn start_auto_save_task(&self, interval_ms: u64) {
        let data = Arc::clone(&self.data);
        let last_save_time = Arc::clone(&self.last_save_time);
        let should_stop = Arc::clone(&self.should_stop);
        let base_dir = self.base_dir.clone();
        let config = self.config.clone();
        
        std::thread::spawn(move || {
            let interval = std::time::Duration::from_millis(interval_ms);
            
            while !*should_stop.lock().unwrap() {
                std::thread::sleep(interval);
                
                let should_save = {
                    let last_save = *last_save_time.lock().unwrap();
                    last_save.elapsed() >= interval
                };
                
                if should_save {
                    if let Err(e) = Self::save_all_data(&data, &base_dir, &config) {
                        eprintln!("Auto-save failed: {}", e);
                    } else {
                        *last_save_time.lock().unwrap() = Instant::now();
                    }
                }
            }
        });
    }

    /// Save all data (static method for use in background thread)
    fn save_all_data(
        data: &Arc<Mutex<HashMap<String, BTreeMap<u64, Measurement>>>>,
        base_dir: &Path,
        config: &LogfileConfig,
    ) -> Result<()> {
        let data_lock = data.lock().unwrap();
        
        for (sensor_name, measurements) in data_lock.iter() {
            let file_path = base_dir.join(format!("{}.dat", sensor_name));
            Self::save_sensor_data_static(measurements, &file_path, config)?;
        }
        
        Ok(())
    }

    /// Save sensor data to file (static method)
    fn save_sensor_data_static(
        measurements: &BTreeMap<u64, Measurement>,
        file_path: &Path,
        config: &LogfileConfig,
    ) -> Result<()> {
        let measurements_vec: Vec<_> = measurements.values().cloned().collect();
        
        match config.save_format {
            SaveFormat::Binary => {
                let data = bincode::serialize(&measurements_vec)?;
                std::fs::write(file_path, data)?;
            }
            SaveFormat::Json => {
                let json = serde_json::to_string_pretty(&measurements_vec)?;
                std::fs::write(file_path, json)?;
            }
            SaveFormat::Csv => {
                let mut csv_content = String::from("timestamp,data\n");
                for measurement in &measurements_vec {
                    csv_content.push_str(&format!("{},{}\n", measurement.time, measurement.data));
                }
                std::fs::write(file_path, csv_content)?;
            }
        }
        
        Ok(())
    }

    /// Get file path for a sensor
    fn get_sensor_file_path(&self, sensor_name: &str) -> PathBuf {
        match self.config.save_format {
            SaveFormat::Binary => self.base_dir.join(format!("{}.dat", sensor_name)),
            SaveFormat::Json => self.base_dir.join(format!("{}.json", sensor_name)),
            SaveFormat::Csv => self.base_dir.join(format!("{}.csv", sensor_name)),
        }
    }

    /// Save sensor data to file
    fn save_sensor_data(
        &self,
        _sensor_name: &str,
        measurements: &BTreeMap<u64, Measurement>,
        file_path: &Path,
    ) -> Result<()> {
        Self::save_sensor_data_static(measurements, file_path, &self.config)
    }

    /// Load binary data from file
    fn load_binary_data(&self, file_path: &Path) -> Result<Vec<Measurement>> {
        let data = std::fs::read(file_path)?;
        let measurements: Vec<Measurement> = bincode::deserialize(&data)?;
        Ok(measurements)
    }

    /// Load JSON data from file
    fn load_json_data(&self, file_path: &Path) -> Result<Vec<Measurement>> {
        let json = std::fs::read_to_string(file_path)?;
        let measurements: Vec<Measurement> = serde_json::from_str(&json)?;
        Ok(measurements)
    }

    /// Load CSV data from file
    fn load_csv_data(&self, file_path: &Path) -> Result<Vec<Measurement>> {
        let csv_content = std::fs::read_to_string(file_path)?;
        let mut measurements = Vec::new();
        
        for (i, line) in csv_content.lines().enumerate() {
            if i == 0 {
                continue; // Skip header
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                if let Ok(timestamp) = parts[0].parse::<u64>() {
                    let data = parts[1..].join(","); // Handle data containing commas
                    measurements.push(Measurement::with_timestamp(timestamp, data));
                }
            }
        }
        
        Ok(measurements)
    }

    /// Trim RAM data to keep only recent measurements
    fn trim_ram_data(&self) {
        let mut data_lock = self.data.lock().unwrap();
        let mut memory_lock = self.memory_usage.lock().unwrap();
        
        for (_sensor_name, sensor_data) in data_lock.iter_mut() {
            let count = sensor_data.len();
            if count > 1000 {
                // Keep only the most recent 1000 measurements
                let keys_to_remove: Vec<_> = sensor_data.keys().take(count - 1000).copied().collect();
                for key in keys_to_remove {
                    if let Some(removed) = sensor_data.remove(&key) {
                        *memory_lock = memory_lock.saturating_sub(estimate_measurement_size(&removed));
                    }
                }
            }
        }
    }
}

impl Drop for SensorlogRam {
    fn drop(&mut self) {
        // Stop background task
        *self.should_stop.lock().unwrap() = true;
        
        // Save data one last time
        if let Err(e) = self.save_to_disk() {
            eprintln!("Failed to save data on drop: {}", e);
        }
    }
}

/// Estimate memory size of a measurement
fn estimate_measurement_size(measurement: &Measurement) -> usize {
    std::mem::size_of::<u64>() + measurement.data.len() + std::mem::size_of::<String>()
}

// For compatibility with the original sensorlog API
pub type Sensorlog = SensorlogRam;