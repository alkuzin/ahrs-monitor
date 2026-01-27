// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application state module.

use tsilna_nav::protocol::idtp::IdtpFrame;

#[derive(Debug, Default)]
pub struct FrameContext {
    /// IMU Data Transfer Protocol frame.
    pub frame: Option<IdtpFrame>,
    /// Total number of packets.
    pub total_packets: usize,
    /// Number of broken packets.
    pub bad_packets: usize,
}

/// Application events enumeration.
#[derive(Debug)]
pub enum AppEvent {
    /// Event for updating IMU connection status.
    UpdateConnectionStatus(bool),
    /// Event for handling received frame.
    FrameReceived(FrameContext),
}
