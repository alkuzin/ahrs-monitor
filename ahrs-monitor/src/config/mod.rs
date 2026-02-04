// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application's configurations.

mod imu;
mod net;

use crate::app_config;
pub use imu::*;
pub use net::*;
use serde::{Deserialize, Serialize};

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

app_config! {
    /// Application's configurations struct.
    pub struct AppConfig {
        /// IMU configurations.
        pub imu: ImuConfig,
        /// Networks configurations.
        pub net: NetConfig,
    }
}
