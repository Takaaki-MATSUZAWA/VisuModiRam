use serde::{Deserialize, Serialize};

/// Storage quota settings for limiting memory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageQuota {
    /// No limit on storage
    Unlimited,
    /// Limit by number of measurements per sensor
    MaxMeasurements(usize),
    /// Limit by memory usage in bytes (approximate)
    MaxMemoryBytes(usize),
    /// Limit by time window in milliseconds
    MaxTimeWindow(u64),
}

impl Default for StorageQuota {
    fn default() -> Self {
        StorageQuota::MaxMeasurements(10000)
    }
}

impl StorageQuota {
    /// Check if the quota allows adding a new measurement
    pub fn can_add(&self, current_count: usize, _current_memory: usize, oldest_time: Option<u64>, newest_time: u64) -> bool {
        match self {
            StorageQuota::Unlimited => true,
            StorageQuota::MaxMeasurements(max) => current_count < *max,
            StorageQuota::MaxMemoryBytes(max) => _current_memory < *max,
            StorageQuota::MaxTimeWindow(window) => {
                if let Some(oldest) = oldest_time {
                    newest_time - oldest <= *window
                } else {
                    true
                }
            }
        }
    }

    /// Calculate how many measurements should be removed to comply with quota
    pub fn measurements_to_remove(&self, current_count: usize, _current_memory: usize, oldest_time: Option<u64>, newest_time: u64) -> usize {
        match self {
            StorageQuota::Unlimited => 0,
            StorageQuota::MaxMeasurements(max) => {
                if current_count > *max {
                    current_count - max
                } else {
                    0
                }
            }
            StorageQuota::MaxMemoryBytes(_max) => {
                // For simplicity, remove 10% of measurements when memory limit is exceeded
                if !self.can_add(current_count, _current_memory, oldest_time, newest_time) {
                    current_count / 10
                } else {
                    0
                }
            }
            StorageQuota::MaxTimeWindow(window) => {
                if let Some(oldest) = oldest_time {
                    if newest_time - oldest > *window {
                        // Estimate how many measurements are outside the window
                        // This is a rough estimation
                        current_count / 4
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
        }
    }
}