// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application's configurations.

use crate::app_config;
use serde::{Deserialize, Serialize};
use std::{fs, process};
use tsilna_nav::protocol::idtp::payload::{IdtpPayload, Imu6};

/// Window width in pixels.
pub const APP_WINDOW_WIDTH: f32 = 1024.0;

/// Window height in pixels.
pub const APP_WINDOW_HEIGHT: f32 = 768.0;

/// Window size in pixels.
pub const APP_WINDOW_SIZE: [f32; 2] = [APP_WINDOW_WIDTH, APP_WINDOW_HEIGHT];

/// Title of the window.
pub const APP_WINDOW_TITLE: &str = "AHRS Monitor";

/// Project version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: move to config:
/// Ingester UDP IP address.
pub const UDP_IP_ADDR: &str = "127.0.0.1";

/// Ingester UDP port.
pub const UDP_PORT: u16 = 10000;

/// Max number of frame contexts in history.
pub const HISTORY_MAX_SIZE: usize = 32;

/// MPSC channel max number of messages in the buffer.
pub const MPSC_CHANNEL_BUFFER_SIZE: usize = 128;

app_config! {
    /// Application's configurations struct.
    pub struct AppConfig {
        /// IMU configurations.
        pub imu: ImuConfig,
    }
}

app_config! {
    /// IMU configurations.
    pub struct ImuConfig {
        /// IMU sample rate in Hz.
        pub sample_rate: f32,
        /// IDTP payload type.
        pub payload_type: u8,
        #[serde(skip)]
        /// Info about IMU metrics in IDTP payload.
        pub metrics: ImuMetrics,
    }
}

impl ImuConfig {
    /// TODO:
    pub fn is_correct(&self) -> bool {
        let standard_types_range = 0x00..0x06 + 1;
        standard_types_range.contains(&self.payload_type)
    }
}

app_config! {
    #[derive(Copy, Clone)]
    /// Info about IMU metrics in IDTP payload to handle.
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

impl From<u8> for ImuMetrics {
    /// Get IMU metrics based on payload type.
    ///
    /// # Parameters
    /// - `payload_type` - given payload type to handle.
    ///
    /// # Returns
    /// - IMU metrics from payload type.
    fn from(payload_type: u8) -> Self {
        match payload_type {
            0x00 => Self {
                acc: true,
                ..Self::default()
            },
            0x01 => Self {
                gyr: true,
                ..Self::default()
            },
            0x02 => Self {
                mag: true,
                ..Self::default()
            },
            0x03 => Self {
                acc: true,
                gyr: true,
                ..Self::default()
            },
            0x04 => Self {
                acc: true,
                gyr: true,
                mag: true,
                ..Self::default()
            },
            0x05 => Self {
                acc: true,
                gyr: true,
                mag: true,
                baro: true,
                ..Self::default()
            },
            0x06 => Self {
                quat: true,
                ..Self::default()
            },
            _ => Self::default(),
        }
    }
}

/// Load application's configurations from specified path.
///
/// # Parameters
/// - `path` - given config file path.
///
/// # Returns
/// - Application's configurations.
pub fn load_config(path: &str) -> AppConfig {
    let content = fs::read_to_string(path).unwrap_or_else(|err| {
        log::error!("Error load config '{}': {}", path, err);
        process::exit(1);
    });

    let mut config: AppConfig =
        toml::from_str(&content).unwrap_or_else(|err| {
            log::error!("Error to parse TOML: {}", err);
            process::exit(1);
        });

    config.imu.metrics = config.imu.payload_type.into();
    config
}
