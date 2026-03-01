// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Inertial Measurement Unit (IMU) configurations.

use indtp::payload::PayloadType;
use crate::{
    app_config,
    config::{Deserialize, Serialize},
};

app_config! {
    /// IMU configurations.
    pub struct ImuConfig {
        /// IMU sample rate in Hz.
        pub sample_rate: f32,
        /// INDTP payload type.
        pub payload_type: u8,
        #[serde(skip)]
        /// Info about IMU metrics in IDTP payload.
        pub metrics: ImuMetrics,
        /// IMU device identifier.
        pub device_id: u16,
        /// INDTP protocol mode.
        pub protocol_mode: u8,
    }
}

impl ImuConfig {
    /// Check whether IMU config is correct.
    ///
    /// # Returns
    /// - `true` - if config is correct.
    /// - `false` - otherwise.
    #[must_use]
    #[inline]
    pub fn is_correct(&self) -> bool {
        self.payload_type().is_standard()
    }

    /// Get payload type.
    ///
    /// # Returns
    /// - Payload type according to IDTP specification.
    #[must_use]
    #[inline]
    pub fn payload_type(&self) -> PayloadType {
        PayloadType::from(self.payload_type)
    }
}

app_config! {
    /// Info about IMU metrics in IDTP payload to handle.
    #[derive(Copy)]
    #[allow(clippy::struct_excessive_bools)]
    pub struct ImuMetrics {
        /// Flag that shows whether accelerometer data is in payload.
        pub acc: bool,
        /// Flag that shows whether gyroscope data is in payload.
        pub gyr: bool,
        /// Flag that shows whether magnetometer data is in payload.
        pub mag: bool,
        /// Flag that shows whether barometer data is in payload.
        pub baro: bool,
        /// Flag that shows whether attitude (quaternion) data is in payload.
        pub quat: bool,
    }
}

impl From<PayloadType> for ImuMetrics {
    /// Get IMU metrics based on payload type.
    ///
    /// # Parameters
    /// - `payload_type` - given payload type to handle.
    ///
    /// # Returns
    /// - IMU metrics from payload type.
    fn from(payload_type: PayloadType) -> Self {
        match payload_type {
            PayloadType::Imu3Acc => Self {
                acc: true,
                ..Self::default()
            },
            PayloadType::Imu3Gyr => Self {
                gyr: true,
                ..Self::default()
            },
            PayloadType::Imu3Mag => Self {
                mag: true,
                ..Self::default()
            },
            PayloadType::Imu6 => Self {
                acc: true,
                gyr: true,
                ..Self::default()
            },
            PayloadType::Imu9 => Self {
                acc: true,
                gyr: true,
                mag: true,
                ..Self::default()
            },
            PayloadType::Imu10 => Self {
                acc: true,
                gyr: true,
                mag: true,
                baro: true,
                ..Self::default()
            },
            PayloadType::ImuQuat => Self {
                quat: true,
                ..Self::default()
            },
            PayloadType::Reserved(_) => unreachable!(
                "PayloadType::Reserved should never be used"
            ),
        }
    }
}
