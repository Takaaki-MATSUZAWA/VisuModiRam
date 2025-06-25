use sensorlog_ram::{SensorlogRam, LogfileConfig, quota::StorageQuota, logfile_config::SaveFormat};
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    // Configure the logger
    let mut config = LogfileConfig::new();
    config.set_default_storage_quota(StorageQuota::MaxMeasurements(1000));
    config.set_auto_save_interval(Some(5000)); // Save every 5 seconds
    config.set_save_format(SaveFormat::Json); // Use JSON for this example
    
    // Create the logger
    let logger = SensorlogRam::new("./example_logs", config)?;
    
    println!("Starting sensor data collection...");
    
    // Simulate sensor data collection
    let start_time = Instant::now();
    for i in 0..50 {
        // Simulate temperature sensor
        let temp_value = 20.0 + (i as f64 * 0.1) + (i as f64).sin() * 2.0;
        logger.store_measurement(None, "temperature", &temp_value.to_string())?;
        
        // Simulate pressure sensor
        let pressure_value = 1013.25 + (i as f64 * 0.05) + (i as f64 * 0.1).cos() * 5.0;
        logger.store_measurement(None, "pressure", &pressure_value.to_string())?;
        
        // Simulate humidity sensor
        let humidity_value = 50.0 + (i as f64 * 0.2).sin() * 10.0;
        logger.store_measurement(None, "humidity", &humidity_value.to_string())?;
        
        if i % 10 == 0 {
            println!("Collected {} data points", i * 3);
            println!("Memory usage: {} bytes", logger.get_memory_usage());
            
            // Show sensor names
            let sensors = logger.get_sensor_names();
            println!("Active sensors: {:?}", sensors);
        }
        
        // Small delay to simulate real sensor timing
        std::thread::sleep(Duration::from_millis(100));
    }
    
    println!("\nData collection completed!");
    
    // Demonstrate data retrieval
    println!("\nRetrieving recent temperature data:");
    let recent_temp = logger.fetch_measurements("temperature", None, None, Some(10))?;
    for measurement in recent_temp {
        println!("  Time: {}ms, Temp: {}°C", measurement.time, measurement.data);
    }
    
    // Demonstrate time-range query
    let collection_duration = start_time.elapsed().as_millis() as u64;
    let mid_point = collection_duration / 2;
    
    println!("\nRetrieving pressure data from middle of collection:");
    let mid_pressure = logger.fetch_measurements(
        "pressure", 
        Some(mid_point),
        Some(mid_point + 1000), // 1 second window
        None
    )?;
    for measurement in mid_pressure {
        println!("  Time: {}ms, Pressure: {} hPa", measurement.time, measurement.data);
    }
    
    // Manually save to disk
    println!("\nSaving all data to disk...");
    logger.save_to_disk()?;
    
    // Show final statistics
    println!("\nFinal statistics:");
    for sensor in logger.get_sensor_names() {
        let count = logger.get_measurement_count(&sensor);
        println!("  {}: {} measurements", sensor, count);
    }
    
    println!("Total memory usage: {} bytes", logger.get_memory_usage());
    println!("Data saved to: ./example_logs/");
    
    Ok(())
}