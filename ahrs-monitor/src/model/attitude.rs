// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Declarations for orientation of the body in 3D space.

use nalgebra::UnitQuaternion;

/// Represents the orientation using the Euler angle convention.
pub struct Attitude {
    /// Rotation around X-axis in **radians**.
    pub roll: f32,
    /// Rotation around Y-axis in **radians**.
    pub pitch: f32,
    /// Rotation around Z-axis in **radians**.
    pub yaw: f32,
}

impl Attitude {
    /// Construct new `Attitude` object.
    ///
    /// # Parameters
    /// - `roll` - given rotation around X-axis in **radians**.
    /// - `pitch` - given rotation around Y-axis in **radians**.
    /// - `yaw` - given rotation around Z-axis in **radians**.
    ///
    /// # Returns
    /// - New `Attitude` object.
    #[must_use]
    pub const fn new(roll: f32, pitch: f32, yaw: f32) -> Self {
        Self { roll, pitch, yaw }
    }

    /// Construct new `Attitude` object from given quaternion.
    ///
    /// # Parameters
    /// - `q` - given quaternion to construct from.
    ///
    /// # Returns
    /// - New `Attitude` object.
    #[inline]
    #[must_use]
    pub fn from_quaternion(q: &UnitQuaternion<f32>) -> Self {
        let (roll, pitch, yaw) = q.euler_angles();
        Self { roll, pitch, yaw }
    }

    /// Convert the attitude to quaternion.
    ///
    /// # Returns
    /// - New unit quaternion from Euler angles.
    #[inline]
    #[must_use]
    pub fn to_quaternion(&self) -> UnitQuaternion<f32> {
        UnitQuaternion::from_euler_angles(self.roll, self.pitch, self.yaw)
    }
}
