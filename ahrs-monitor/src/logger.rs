// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU data logger implementation.

use tsilna_nav::protocol::idtp::payload::{Imu10, Imu3Acc, Imu3Gyr, Imu3Mag, Imu6, Imu9, ImuQuat};

/// IMU data log record.
pub struct LogRecord {
    /// Timestamp represents the sensor-local time.
    pub timestamp: u32,
    /// Vendor-specific unique IMU device identifier.
    pub device_id: u16,
    /// Accelerometer readings.
    pub acc: Option<[f32; 3]>,
    /// Gyroscope readings.
    pub gyr: Option<[f32; 3]>,
    /// Magnetometer readings.
    pub mag: Option<[f32; 3]>,
    /// Barometer readings.
    pub baro: Option<f32>,
    /// Attitude (Quaternion).
    pub quat: [f32; 4],
    /// Attitude (Euler angles).
    pub euler: [f32; 3],
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
        let acc_data = [self.acc_x, self.acc_y, self.acc_z];
        record.acc = Some(acc_data);
    }
}

impl ToLog for Imu3Gyr {
    fn fill_record(&self, record: &mut LogRecord) {
        let gyr_data = [self.gyr_x, self.gyr_y, self.gyr_z];
        record.gyr = Some(gyr_data);
    }
}

impl ToLog for Imu3Mag {
    fn fill_record(&self, record: &mut LogRecord) {
        let mag_data = [self.mag_x, self.mag_y, self.mag_z];
        record.mag = Some(mag_data);
    }
}

impl ToLog for Imu6 {
    fn fill_record(&self, record: &mut LogRecord) {
        let acc_data = [self.acc.acc_x, self.acc.acc_y, self.acc.acc_z];
        let gyr_data = [self.gyr.gyr_x, self.gyr.gyr_y, self.gyr.gyr_z];

        record.acc = Some(acc_data);
        record.gyr = Some(gyr_data);
    }
}

impl ToLog for Imu9 {
    fn fill_record(&self, record: &mut LogRecord) {
        let acc_data = [self.acc.acc_x, self.acc.acc_y, self.acc.acc_z];
        let gyr_data = [self.gyr.gyr_x, self.gyr.gyr_y, self.gyr.gyr_z];
        let mag_data = [self.mag.mag_x, self.mag.mag_y, self.mag.mag_z];

        record.acc = Some(acc_data);
        record.gyr = Some(gyr_data);
        record.mag = Some(mag_data);
    }
}

impl ToLog for Imu10 {
    fn fill_record(&self, record: &mut LogRecord) {
        let acc_data = [self.acc.acc_x, self.acc.acc_y, self.acc.acc_z];
        let gyr_data = [self.gyr.gyr_x, self.gyr.gyr_y, self.gyr.gyr_z];
        let mag_data = [self.mag.mag_x, self.mag.mag_y, self.mag.mag_z];

        record.acc = Some(acc_data);
        record.gyr = Some(gyr_data);
        record.mag = Some(mag_data);
        record.baro = Some(self.baro);
    }
}

impl ToLog for ImuQuat {
    fn fill_record(&self, record: &mut LogRecord) {
        record.quat = [self.w, self.x, self.y, self.z];
    }
}
