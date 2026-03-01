// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU data logger implementation.

use crate::config::AppConfig;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use indtp::payload::{
    Imu3Acc, Imu3Gyr, Imu3Mag, Imu6, Imu9, Imu10, ImuQuat,
};

#[derive(Debug, Default, Serialize, Deserialize)]
/// IMU data log record.
pub struct LogRecord {
    /// Timestamp represents the sensor-local time.
    pub timestamp: u32,
    /// Vendor-specific unique IMU device identifier.
    pub device_id: u8,
    /// Accelerometer reading along X-axis.
    pub acc_x: Option<f32>,
    /// Accelerometer reading along for Y-axis.
    pub acc_y: Option<f32>,
    /// Accelerometer reading along for Z-axis.
    pub acc_z: Option<f32>,
    /// Gyroscope reading along the X-axis.
    pub gyr_x: Option<f32>,
    /// Gyroscope reading along the Y-axis.
    pub gyr_y: Option<f32>,
    /// Gyroscope reading along the Z-axis.
    pub gyr_z: Option<f32>,
    /// Magnetometer reading along the X-axis.
    pub mag_x: Option<f32>,
    /// Magnetometer reading along the Y-axis.
    pub mag_y: Option<f32>,
    /// Magnetometer reading along the Z-axis.
    pub mag_z: Option<f32>,
    /// Barometer reading.
    pub pressure: Option<f32>,
    /// Quaternion scalar component W.
    pub q_w: f32,
    /// Quaternion vector component X.
    pub q_x: f32,
    /// Quaternion vector component Y.
    pub q_y: f32,
    /// Quaternion vector component Z.
    pub q_z: f32,
    /// Rotation around X-axis.
    pub roll: f32,
    /// Rotation around Y-axis.
    pub pitch: f32,
    /// Rotation around Z-axis.
    pub yaw: f32,
}

/// Trait for logging IDTP frame payload data.
pub trait ToLog {
    /// Fill IMU data log record.
    ///
    /// # Parameters
    /// - `record` - given IMU data log record to fill.
    fn fill_record(&self, record: &mut LogRecord);
}

impl ToLog for Imu3Acc {
    fn fill_record(&self, record: &mut LogRecord) {
        record.acc_x = Some(self.acc_x.get());
        record.acc_y = Some(self.acc_y.get());
        record.acc_z = Some(self.acc_z.get());
    }
}

impl ToLog for Imu3Gyr {
    fn fill_record(&self, record: &mut LogRecord) {
        record.gyr_x = Some(self.gyr_x.get());
        record.gyr_y = Some(self.gyr_y.get());
        record.gyr_z = Some(self.gyr_z.get());
    }
}

impl ToLog for Imu3Mag {
    fn fill_record(&self, record: &mut LogRecord) {
        record.mag_x = Some(self.mag_x.get());
        record.mag_y = Some(self.mag_y.get());
        record.mag_z = Some(self.mag_z.get());
    }
}

impl ToLog for Imu6 {
    fn fill_record(&self, record: &mut LogRecord) {
        record.acc_x = Some(self.acc.acc_x.get());
        record.acc_y = Some(self.acc.acc_y.get());
        record.acc_z = Some(self.acc.acc_z.get());
        record.gyr_x = Some(self.gyr.gyr_x.get());
        record.gyr_y = Some(self.gyr.gyr_y.get());
        record.gyr_z = Some(self.gyr.gyr_z.get());
    }
}

impl ToLog for Imu9 {
    fn fill_record(&self, record: &mut LogRecord) {
        record.acc_x = Some(self.acc.acc_x.get());
        record.acc_y = Some(self.acc.acc_y.get());
        record.acc_z = Some(self.acc.acc_z.get());
        record.gyr_x = Some(self.gyr.gyr_x.get());
        record.gyr_y = Some(self.gyr.gyr_y.get());
        record.gyr_z = Some(self.gyr.gyr_z.get());
        record.mag_x = Some(self.mag.mag_x.get());
        record.mag_y = Some(self.mag.mag_y.get());
        record.mag_z = Some(self.mag.mag_z.get());
    }
}

impl ToLog for Imu10 {
    fn fill_record(&self, record: &mut LogRecord) {
        record.acc_x = Some(self.acc.acc_x.get());
        record.acc_y = Some(self.acc.acc_y.get());
        record.acc_z = Some(self.acc.acc_z.get());
        record.gyr_x = Some(self.gyr.gyr_x.get());
        record.gyr_y = Some(self.gyr.gyr_y.get());
        record.gyr_z = Some(self.gyr.gyr_z.get());
        record.mag_x = Some(self.mag.mag_x.get());
        record.mag_y = Some(self.mag.mag_y.get());
        record.mag_z = Some(self.mag.mag_z.get());
        record.pressure = Some(self.baro.get());
    }
}

impl ToLog for ImuQuat {
    fn fill_record(&self, record: &mut LogRecord) {
        record.q_w = self.w.get();
        record.q_x = self.x.get();
        record.q_y = self.y.get();
        record.q_z = self.z.get();
    }
}

/// IMU data log records handler.
pub struct Logger {
    /// CSV file writer.
    writer: csv::Writer<fs::File>,
    /// Path to log file.
    path: String,
    /// Recording start timestamp.
    start_time: std::time::Instant,
}

impl Logger {
    /// Construct new `Logger` object.
    ///
    /// # Parameters
    /// - `cfg` - given application's config to handle.
    ///
    /// # Returns
    /// - New `Logger` object - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - I/O errors.
    /// - Error to create log file.
    pub fn new(cfg: &AppConfig) -> std::io::Result<Self> {
        fs::create_dir_all(&cfg.log.directory)?;

        let now = Local::now();
        let filename = format!("log_{}.csv", now.format("%d-%m-%Y_%H-%M-%S"));

        let mut path = PathBuf::from(&cfg.log.directory);
        path.push(filename);

        let path_str = path.to_string_lossy().into_owned();
        let file = fs::File::create(&path)?;

        Ok(Self {
            writer: csv::Writer::from_writer(file),
            path: path_str,
            start_time: std::time::Instant::now(),
        })
    }

    /// Get log file path.
    ///
    /// # Returns
    /// - Relative log file path.
    #[must_use]
    pub const fn path(&self) -> &String {
        &self.path
    }

    /// Write record into the log file.
    ///
    /// # Parameters
    /// - `record` - given IMU data log record to handle.
    ///
    /// # Returns
    /// - `Ok` - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - I/O errors.
    /// - CSV file handling errors.
    pub fn write(&mut self, record: &LogRecord) -> anyhow::Result<()> {
        self.writer.serialize(record)?;
        self.writer.flush()?;

        Ok(())
    }

    /// Get timestamp since the start of the recording.
    ///
    /// # Returns
    /// - Timestamp in string representation.
    #[must_use]
    pub fn timestamp_str(&self) -> String {
        let elapsed = self.start_time.elapsed();
        let secs = elapsed.as_secs();
        let seconds = secs % 60;
        let minutes = (secs / 60) % 60;
        let hours = secs / 3600;

        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }
}
