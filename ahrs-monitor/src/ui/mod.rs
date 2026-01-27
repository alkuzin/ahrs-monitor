// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! The core responsible for AHRS Monitor user interface.

pub mod inspector;

/// Application tabs enumeration.
#[derive(Debug, Default, PartialEq)]
pub enum Tab {
    /// Tab for displaying 3D model.
    #[default]
    Dashboard,
    /// Sensor readings plots.
    Telemetry,
    /// Tab for displaying raw packet inspector.
    Inspector,
}
