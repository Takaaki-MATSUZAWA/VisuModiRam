# sensorlog-ram

A high-performance RAM-based logging system compatible with the sensorlog interface, designed for VisuModiRam.

## Features

- **RAM-based storage**: Ultra-fast data logging with measurements stored in memory
- **Periodic disk persistence**: Automatic background saving to disk with configurable intervals
- **Compatible API**: Drop-in replacement for sensorlog with the same interface
- **Flexible storage quotas**: Control memory usage with various quota strategies
- **Multiple save formats**: Binary, JSON, and CSV export formats
- **Thread-safe**: Safe concurrent access from multiple threads
- **Memory management**: Automatic cleanup based on configurable limits

## Usage

```rust
use sensorlog_ram::{SensorlogRam, LogfileConfig, quota::StorageQuota};

// Create configuration
let mut config = LogfileConfig::new();
config.set_default_storage_quota(StorageQuota::MaxMeasurements(10000));
config.set_auto_save_interval(Some(30000)); // Save every 30 seconds

// Create logger
let logger = SensorlogRam::new("./logs", config)?;

// Store measurements
logger.store_measurement(None, "temperature", "25.5")?;
logger.store_measurement(None, "pressure", "1013.25")?;

// Fetch measurements
let measurements = logger.fetch_measurements(
    "temperature", 
    None,      // time_start
    None,      // time_end  
    Some(100)  // limit
)?;

// Save to disk manually
logger.save_to_disk()?;
```

## API Compatibility

This crate provides a compatible interface with the original sensorlog:

- `store_measurement(timestamp, sensor_name, data)` - Store a measurement
- `fetch_measurements(sensor_name, time_start, time_end, limit)` - Retrieve measurements
- Automatic quota management and persistence

## Storage Quotas

Control memory usage with flexible quota strategies:

```rust
use sensorlog_ram::quota::StorageQuota;

// Unlimited storage (RAM only)
StorageQuota::Unlimited

// Limit by number of measurements
StorageQuota::MaxMeasurements(10000)

// Limit by memory usage (approximate)
StorageQuota::MaxMemoryBytes(50 * 1024 * 1024) // 50MB

// Limit by time window
StorageQuota::MaxTimeWindow(60 * 60 * 1000) // 1 hour in milliseconds
```

## Save Formats

Choose from multiple formats for disk persistence:

- **Binary**: Fast, compact using bincode
- **JSON**: Human-readable, slower
- **CSV**: Spreadsheet compatible

## Configuration

```rust
use sensorlog_ram::{LogfileConfig, logfile_config::SaveFormat};

let mut config = LogfileConfig::new();
config.set_compression(true);                    // Enable compression
config.set_auto_save_interval(Some(60000));      // Auto-save every minute
config.set_max_ram_usage(100 * 1024 * 1024);    // 100MB RAM limit
config.set_save_format(SaveFormat::Binary);      // Use binary format
```

## Memory Management

The system automatically manages memory usage:

- Configurable RAM limits with automatic cleanup
- Background saving prevents data loss
- Quota-based measurement removal (oldest first)
- Memory usage estimation and monitoring

## Thread Safety

All operations are thread-safe using internal mutexes:

```rust
use std::sync::Arc;

let logger = Arc::new(SensorlogRam::new("./logs", config)?);

// Safe to use from multiple threads
let logger_clone = Arc::clone(&logger);
std::thread::spawn(move || {
    logger_clone.store_measurement(None, "sensor1", "value")?;
});
```

## Performance

Optimized for high-frequency logging:

- In-memory storage for maximum speed
- Minimal allocation during measurement storage
- Efficient time-based indexing using BTreeMap
- Background I/O to avoid blocking measurement collection