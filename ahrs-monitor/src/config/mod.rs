// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application's configurations.

mod imu;
mod logging;
mod net;

use crate::{app_config, config::logging::LoggingConfig};
pub use imu::*;
use indtp::payload::PayloadType;
pub use net::*;
use serde::{Deserialize, Serialize};
use std::fs;
use indtp::types::{AesKey, HmacKey};

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
pub const AES_KEY: &AesKey =
    include_bytes!("../../configs/firmware/secrets/aes.key");

/// HMAC-SHA256 key.
pub const HMAC_KEY: &HmacKey =
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
/// - Application's configurations - in case of success.
/// - `Err` - otherwise.
pub fn load_config(path: &str) -> anyhow::Result<AppConfig> {
    let content = fs::read_to_string(path)?;
    let mut config: AppConfig = toml::from_str(&content)?;

    let payload_type = PayloadType::from(config.imu.payload_type);
    config.imu.metrics = ImuMetrics::from(payload_type);

    Ok(config)
}
