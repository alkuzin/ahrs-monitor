// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application logging related configurations.

use serde::{Deserialize, Serialize};

app_config! {
    /// Logging configurations struct.
    pub struct LoggingConfig {
        /// Directory where logs are stored.
        pub directory: String,
    }
}
