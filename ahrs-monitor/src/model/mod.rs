// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application state module.

/// Application events enumeration.
#[derive(Debug)]
pub enum AppEvent {
    /// Event for updating IMU connection status.
    UpdateConnectionStatus(bool),
}
