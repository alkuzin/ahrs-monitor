// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application state module.

use crate::core::StandardPayload;
use indtp::{Flags, Header};
use tsilna_nav::math::Quat32;

/// TODO:
#[derive(Default, Debug)]
pub struct FrameWrapper {
    /// TODO:
    pub header: Header,
    /// TODO:
    pub payload: Option<StandardPayload>,
    /// TODO:
    pub trailer: Vec<u8>,
    /// TODO:
    pub size: usize,
    /// TODO:
    pub flags: Flags,
}

/// Context data after receiving the frame.
#[derive(Default, Debug)]
pub struct FrameContext {
    /// INDTP frame.
    pub frame: Option<FrameWrapper>,
    /// Sensor-local time in microseconds.
    pub timestamp: u32,
    /// Indicator whether current frame is valid.
    pub is_valid: bool,
    /// Total number of packets.
    pub total_packets: usize,
    /// Number of broken packets.
    pub bad_packets: usize,
    /// Number of packets per second.
    pub pps: usize,
    /// Unit for representation of rotation in space.
    pub quaternion: Option<Quat32>,
}

/// Application events enumeration.
pub enum AppEvent {
    /// Event for updating IMU connection status.
    UpdateConnectionStatus(bool),
    /// Event for handling received frame.
    FrameReceived(Box<FrameContext>),
}
