# Migration from sensorlog to sensorlog-ram

This guide shows how to migrate from the original `sensorlog` crate to `sensorlog-ram` for better performance in VisuModiRam.

## Key Differences

| Feature | sensorlog | sensorlog-ram |
|---------|-----------|---------------|
| Storage | Disk-based | RAM-based with periodic disk saves |
| Performance | Slower I/O operations | Ultra-fast in-memory operations |
| Memory Usage | Low | Higher (configurable limits) |
| Data Persistence | Immediate | Background saves + manual save |
| API Compatibility | Original | Compatible interface |

## Basic Migration

### Before (sensorlog)
```rust
use sensorlog::{logfile_config::LogfileConfig, quota::StorageQuota, Sensorlog};

let mut config = LogfileConfig::new();
config.set_default_storage_quota(StorageQuota::Unlimited);

let logger = Sensorlog::new("./logs", config)?;
logger.store_measurement(None, "sensor1", "123.45")?;
let measurements = logger.fetch_measurements("sensor1", None, None, Some(100))?;
```

### After (sensorlog-ram)
```rust
use sensorlog_ram::{SensorlogRam, LogfileConfig, quota::StorageQuota};

let mut config = LogfileConfig::new();
config.set_default_storage_quota(StorageQuota::Unlimited);

let logger = SensorlogRam::new("./logs", config)?;
logger.store_measurement(None, "sensor1", "123.45")?;
let measurements = logger.fetch_measurements("sensor1", None, None, Some(100))?;
```

## Changes Required

1. **Import statements**: Change `sensorlog::` to `sensorlog_ram::`
2. **Type names**: Change `Sensorlog` to `SensorlogRam`
3. **Configuration**: Add new RAM-specific configuration options

## Enhanced Configuration

sensorlog-ram offers additional configuration options:

```rust
use sensorlog_ram::{SensorlogRam, LogfileConfig, quota::StorageQuota, logfile_config::SaveFormat};

let mut config = LogfileConfig::new();

// Memory management
config.set_max_ram_usage(100 * 1024 * 1024); // 100MB limit
config.set_default_storage_quota(StorageQuota::MaxMeasurements(10000));

// Auto-save configuration
config.set_auto_save_interval(Some(30000)); // Save every 30 seconds
config.set_compression(true); // Enable compression

// Save format
config.set_save_format(SaveFormat::Binary); // Fast binary format
// config.set_save_format(SaveFormat::Json);   // Human-readable
// config.set_save_format(SaveFormat::Csv);    // Spreadsheet compatible

let logger = SensorlogRam::new("./logs", config)?;
```

## New Features Available

### 1. Memory Usage Monitoring
```rust
let memory_bytes = logger.get_memory_usage();
println!("Current RAM usage: {} bytes", memory_bytes);
```

### 2. Manual Save Control
```rust
// Save all data to disk immediately
logger.save_to_disk()?;

// Load previously saved data
let old_data = logger.load_from_disk("sensor1")?;
```

### 3. RAM Management
```rust
// Clear specific sensor data from RAM (keeps disk data)
logger.clear_sensor_ram("sensor1");

// Clear all RAM data
logger.clear_ram();
```

### 4. Enhanced Quota Control
```rust
// Set individual sensor quotas
logger.set_sensor_quota("high_freq_sensor", StorageQuota::MaxMeasurements(50000));
logger.set_sensor_quota("low_freq_sensor", StorageQuota::MaxMeasurements(1000));

// Time-based quotas
logger.set_sensor_quota("recent_only", StorageQuota::MaxTimeWindow(60 * 1000)); // 1 minute
```

## Performance Considerations

### Memory Usage
- RAM usage grows with number of stored measurements
- Configure `max_ram_usage_bytes` to prevent excessive memory usage
- Use storage quotas to limit per-sensor memory consumption

### Auto-Save Settings
- Shorter intervals = more frequent disk I/O, better data safety
- Longer intervals = better performance, higher risk of data loss
- Disable auto-save (`None`) for maximum performance, manual control

### Storage Quotas
- `MaxMeasurements`: Good for limiting memory per sensor
- `MaxMemoryBytes`: Good for overall memory control
- `MaxTimeWindow`: Good for recent-data-only scenarios
- `Unlimited`: Use with caution, monitor memory usage

## Example: Updating probe_interface.rs

### Before
```rust
use sensorlog::{logfile_config::LogfileConfig, quota, Sensorlog};

fn log_service_default() -> Sensorlog {
    let mut logfile_config = LogfileConfig::new();
    logfile_config.set_default_storage_quota(quota::StorageQuota::Unlimited);
    
    let datadir = PathBuf::from("~/.EguiLog");
    let service = Sensorlog::new(&datadir, logfile_config).expect("failed create logfile");
    service
}
```

### After
```rust
use sensorlog_ram::{SensorlogRam, LogfileConfig, quota, logfile_config::SaveFormat};

fn log_service_default() -> SensorlogRam {
    let mut logfile_config = LogfileConfig::new();
    logfile_config.set_default_storage_quota(quota::StorageQuota::MaxMeasurements(100000));
    logfile_config.set_auto_save_interval(Some(30000)); // 30 seconds
    logfile_config.set_max_ram_usage(200 * 1024 * 1024); // 200MB
    logfile_config.set_save_format(SaveFormat::Binary); // Fast format
    
    let datadir = PathBuf::from("~/.EguiLog");
    let service = SensorlogRam::new(&datadir, logfile_config).expect("failed create logfile");
    service
}
```

## Testing the Migration

1. **Run your existing code** with sensorlog-ram
2. **Monitor memory usage** with `get_memory_usage()`
3. **Verify data persistence** by checking saved files
4. **Test data retrieval** to ensure compatibility
5. **Benchmark performance** to confirm improvements

## Rollback Strategy

If you need to rollback:
1. Change imports back to `sensorlog`
2. Remove RAM-specific configuration options
3. The data format is compatible, so existing logs should work

## Performance Tips

1. **Use binary format** for best performance: `SaveFormat::Binary`
2. **Set reasonable quotas** to prevent memory bloat
3. **Configure auto-save intervals** based on your data importance vs. performance needs
4. **Monitor memory usage** in production
5. **Use time-based quotas** for real-time applications where only recent data matters