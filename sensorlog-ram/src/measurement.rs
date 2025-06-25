use serde::{Deserialize, Serialize};

/// Represents a single measurement data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// Timestamp in milliseconds since Unix epoch
    pub time: u64,
    /// String representation of the measured data
    pub data: String,
}

impl Measurement {
    /// Create a new measurement with current timestamp
    pub fn new(data: String) -> Self {
        Self {
            time: crate::current_timestamp_ms(),
            data,
        }
    }

    /// Create a new measurement with specified timestamp
    pub fn with_timestamp(time: u64, data: String) -> Self {
        Self { time, data }
    }

    /// Parse the data as a specific numeric type
    pub fn parse_as<T>(&self) -> Result<T, T::Err>
    where
        T: std::str::FromStr,
    {
        self.data.parse()
    }

    /// Get the data as f64 if possible
    pub fn as_f64(&self) -> Option<f64> {
        self.data.parse().ok()
    }

    /// Get the data as f32 if possible
    pub fn as_f32(&self) -> Option<f32> {
        self.data.parse().ok()
    }
}