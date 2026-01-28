// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application state module.

pub mod payload;

use tsilna_nav::protocol::idtp::IdtpFrame;

/// Context data after receiving the frame.
#[derive(Debug, Default, Clone)]
pub struct FrameContext {
    /// IMU Data Transfer Protocol frame.
    pub frame: Option<IdtpFrame>,
    /// Raw frame bytes.
    pub raw_frame: Vec<u8>,
    /// Indicator whether current frame is valid.
    pub is_valid: bool,
    /// Total number of packets.
    pub total_packets: usize,
    /// Number of broken packets.
    pub bad_packets: usize,
    /// Number of packets per second.
    pub pps: usize,
}

/// Application events enumeration.
#[derive(Debug)]
pub enum AppEvent {
    /// Event for updating IMU connection status.
    UpdateConnectionStatus(bool),
    /// Event for handling received frame.
    FrameReceived(FrameContext),
}
