use crate::quota::StorageQuota;
use serde::{Deserialize, Serialize};

/// Configuration for log file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogfileConfig {
    /// Default storage quota for new sensors
    pub default_storage_quota: StorageQuota,
    /// Whether to compress data when saving to disk
    pub compress_on_save: bool,
    /// Automatic save interval in milliseconds (None = manual save only)
    pub auto_save_interval_ms: Option<u64>,
    /// Maximum memory usage before forced save (in bytes)
    pub max_ram_usage_bytes: usize,
    /// File format for saving data
    pub save_format: SaveFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SaveFormat {
    /// Binary format using bincode (fast, compact)
    Binary,
    /// JSON format (human readable, slower)
    Json,
    /// CSV format (compatible with spreadsheets)
    Csv,
}

impl Default for LogfileConfig {
    fn default() -> Self {
        Self {
            default_storage_quota: StorageQuota::default(),
            compress_on_save: false,
            auto_save_interval_ms: Some(30000), // 30 seconds
            max_ram_usage_bytes: 100 * 1024 * 1024, // 100MB
            save_format: SaveFormat::Binary,
        }
    }
}

impl LogfileConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the default storage quota
    pub fn set_default_storage_quota(&mut self, quota: StorageQuota) {
        self.default_storage_quota = quota;
    }

    /// Enable or disable compression
    pub fn set_compression(&mut self, enabled: bool) {
        self.compress_on_save = enabled;
    }

    /// Set automatic save interval
    pub fn set_auto_save_interval(&mut self, interval_ms: Option<u64>) {
        self.auto_save_interval_ms = interval_ms;
    }

    /// Set maximum RAM usage before forced save
    pub fn set_max_ram_usage(&mut self, bytes: usize) {
        self.max_ram_usage_bytes = bytes;
    }

    /// Set save format
    pub fn set_save_format(&mut self, format: SaveFormat) {
        self.save_format = format;
    }
}