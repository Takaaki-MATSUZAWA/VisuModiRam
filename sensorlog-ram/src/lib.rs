pub mod measurement;
pub mod sensorlog_ram;
pub mod logfile_config;
pub mod quota;

pub use measurement::Measurement;
pub use sensorlog_ram::SensorlogRam;
pub use logfile_config::LogfileConfig;

// Re-export commonly used types for compatibility
use std::time::{SystemTime, UNIX_EPOCH};

pub type Result<T> = anyhow::Result<T>;

/// Get current timestamp in milliseconds since Unix epoch
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}