// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Computer networks related configurations.

use crate::config::{Deserialize, Serialize};

app_config! {
    /// Networks configurations.
    pub struct NetConfig {
        /// Ingester's IP address.
        pub ip_address: String,
        /// Ingester's UDP port.
        pub udp_port: u16,
    }
}
