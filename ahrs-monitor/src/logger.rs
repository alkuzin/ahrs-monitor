// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU data logger implementation.

use tsilna_nav::protocol::idtp::payload::{Imu10, Imu3Acc, Imu3Gyr, Imu3Mag, Imu6, Imu9, ImuQuat};
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
/// IMU data log record.
pub struct LogRecord {
    /// Timestamp represents the sensor-local time.
    pub timestamp: u32,
    /// Vendor-specific unique IMU device identifier.
    pub device_id: u16,
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
        record.acc_x = Some(self.acc_x);
        record.acc_y = Some(self.acc_y);
        record.acc_z = Some(self.acc_z);
    }
}

impl ToLog for Imu3Gyr {
    fn fill_record(&self, record: &mut LogRecord) {
        record.gyr_x = Some(self.gyr_x);
        record.gyr_y = Some(self.gyr_y);
        record.gyr_z = Some(self.gyr_z);
    }
}

impl ToLog for Imu3Mag {
    fn fill_record(&self, record: &mut LogRecord) {
        record.mag_x = Some(self.mag_x);
        record.mag_y = Some(self.mag_y);
        record.mag_z = Some(self.mag_z);
    }
}

impl ToLog for Imu6 {
    fn fill_record(&self, record: &mut LogRecord) {
        record.acc_x = Some(self.acc.acc_x);
        record.acc_y = Some(self.acc.acc_y);
        record.acc_z = Some(self.acc.acc_z);
        record.gyr_x = Some(self.gyr.gyr_x);
        record.gyr_y = Some(self.gyr.gyr_y);
        record.gyr_z = Some(self.gyr.gyr_z);
    }
}

impl ToLog for Imu9 {
    fn fill_record(&self, record: &mut LogRecord) {
        record.acc_x = Some(self.acc.acc_x);
        record.acc_y = Some(self.acc.acc_y);
        record.acc_z = Some(self.acc.acc_z);
        record.gyr_x = Some(self.gyr.gyr_x);
        record.gyr_y = Some(self.gyr.gyr_y);
        record.gyr_z = Some(self.gyr.gyr_z);
        record.mag_x = Some(self.mag.mag_x);
        record.mag_y = Some(self.mag.mag_y);
        record.mag_z = Some(self.mag.mag_z);
    }
}

impl ToLog for Imu10 {
    fn fill_record(&self, record: &mut LogRecord) {
        record.acc_x = Some(self.acc.acc_x);
        record.acc_y = Some(self.acc.acc_y);
        record.acc_z = Some(self.acc.acc_z);
        record.gyr_x = Some(self.gyr.gyr_x);
        record.gyr_y = Some(self.gyr.gyr_y);
        record.gyr_z = Some(self.gyr.gyr_z);
        record.mag_x = Some(self.mag.mag_x);
        record.mag_y = Some(self.mag.mag_y);
        record.mag_z = Some(self.mag.mag_z);
        record.pressure = Some(self.baro);
    }
}

impl ToLog for ImuQuat {
    fn fill_record(&self, record: &mut LogRecord) {
        record.q_w = self.w;
        record.q_x = self.x;
        record.q_y = self.y;
        record.q_z = self.z;
    }
}

/// IMU data log records handler.
pub struct Logger {
    /// CSV file writer.
    writer: csv::Writer<std::fs::File>,
    /// Path to log file.
    path: String,
    /// Recording start timestamp.
    start_time: std::time::Instant,
}

impl Logger {
    /// Construct new `Logger` object.
    ///
    /// # Returns
    /// - New `Logger` object - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - I/O errors.
    /// - Error to create log file.
    pub fn new() -> std::io::Result<Self> {
        // TODO: set specific logging directory.
        let now = Local::now();
        let path = format!("log_{}.csv", now.format("%d-%m-%Y_%H-%M-%S"));
        let file = std::fs::File::create(&path)?;

        Ok(Self {
            writer: csv::Writer::from_writer(file),
            path,
            start_time: std::time::Instant::now(),
        })
    }

    /// Get log file path.
    pub fn path(&self) -> &String {
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
    pub fn timestamp_str(&self) -> String {
        let elapsed = self.start_time.elapsed();
        let secs = elapsed.as_secs();
        let seconds = secs % 60;
        let minutes = (secs / 60) % 60;
        let hours = secs / 3600;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}
