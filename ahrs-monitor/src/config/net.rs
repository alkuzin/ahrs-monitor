// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Computer networks related configurations.

use crate::config::{Deserialize, Serialize};

app_config! {
    /// Networks configurations.
    pub struct NetConfig {
        /// Monitor's IP address.
        pub monitor_ip: String,
        /// Monitor's UDP port.
        pub monitor_port: u16,
        /// IMU's gateway IP address.
        pub imu_gateway_ip: String,
        /// IMU's gateway SSID.
        pub imu_gateway_ssid: String,
        /// IMU's gateway password.
        pub imu_gateway_password: String,
        /// Simulator's IP address.
        pub simulator_ip: String,
        /// Simulator's UDP port.
        pub simulator_port: u16,
        /// Flag whether to use encryption for IMU data transmission.
        pub use_encryption: bool,
    }
}
