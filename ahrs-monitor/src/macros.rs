// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Macros utilities.

/// Apply some traits for config struct declarations.
#[macro_export]
macro_rules! app_config {
    ($($item:item)*) => {
        $(
            #[derive(Serialize, Deserialize, Debug, Default, Clone)]
            #[serde(deny_unknown_fields)]
            $item
        )*
    };
}
