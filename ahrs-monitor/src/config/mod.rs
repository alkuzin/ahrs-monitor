// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application's configurations.

mod imu;
mod logging;
mod net;

use crate::{app_config, config::logging::LoggingConfig};
pub use imu::*;
pub use net::*;
use serde::{Deserialize, Serialize};
use std::{fs, process};
use tsilna_nav::protocol::idtp::payload::PayloadType;

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

/// Max number of frame contexts in history.
pub const HISTORY_MAX_SIZE: usize = 32;

/// MPSC channel max number of messages in the buffer.
pub const MPSC_CHANNEL_BUFFER_SIZE: usize = 128;

/// AHRS Monitor configuration file path.
pub const CONFIG_FILE_PATH: &str = "configs/config.toml";

/// AES-128 encryption key.
pub const AES_KEY: &[u8; 16] =
    include_bytes!("../../configs/firmware/secrets/aes.key");

/// HMAC-SHA256 key.
pub const HMAC_KEY: &[u8; 32] =
    include_bytes!("../../configs/firmware/secrets/hmac.key");

app_config! {
    /// Application's configurations struct.
    pub struct AppConfig {
        /// IMU configurations.
        pub imu: ImuConfig,
        /// Networks configurations.
        pub net: NetConfig,
        /// Logging configurations.
        pub log: LoggingConfig,
    }
}

/// Load application's configurations from specified path.
///
/// # Parameters
/// - `path` - given config file path.
///
/// # Returns
/// - Application's configurations.
#[must_use]
pub fn load_config(path: &str) -> AppConfig {
    let content = fs::read_to_string(path).unwrap_or_else(|err| {
        log::error!("Error load config '{path}': {err}");
        process::exit(1);
    });

    let mut config: AppConfig =
        toml::from_str(&content).unwrap_or_else(|err| {
            log::error!("Error to parse TOML: {err}");
            process::exit(1);
        });

    if let Ok(payload_type) = PayloadType::try_from(config.imu.payload_type) {
        config.imu.metrics = ImuMetrics::from(payload_type);
    } else {
        log::error!("Error to parse payload type: {}", config.imu.payload_type);
        process::exit(1);
    }

    config
}
