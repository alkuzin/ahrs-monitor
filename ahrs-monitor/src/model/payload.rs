// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IDTP frame payload.

use zerocopy::{FromBytes, IntoBytes, Immutable};

/// IDTP payload struct.
#[derive(Debug, Default, Clone, Copy, IntoBytes, FromBytes, Immutable)]
#[repr(C, packed)]
pub struct Payload {
    /// The value of the projection of the acceleration vector along the
    /// X axis (m/s^2).
    pub acc_x: f32,
    /// The value of the projection of the acceleration vector along the
    /// Y axis (m/s^2).
    pub acc_y: f32,
    /// The value of the projection of the acceleration vector along the
    /// Z axis (m/s^2).
    pub acc_z: f32,
    /// Angular velocity along the X axis (rad/s).
    pub gyr_x: f32,
    /// Angular velocity along the Y axis (rad/s).
    pub gyr_y: f32,
    /// Angular velocity along the Z axis (rad/s).
    pub gyr_z: f32,
    /// Magnetometer value along the X axis (Gauss).
    pub mag_x: f32,
    /// Magnetometer value along the Y axis (Gauss).
    pub mag_y: f32,
    /// Magnetometer value along the Z axis (Gauss).
    pub mag_z: f32,
}

/// Payload size in bytes.
pub const PAYLOAD_SIZE: usize = size_of::<Payload>();
