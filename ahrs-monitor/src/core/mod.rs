// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! AHRS Monitor core main module.
//! The core responsible for handling IDTP frames.

pub mod attitude;
mod ingester;

use indtp::payload::PayloadType;
use indtp::{
    payload::{Imu3Acc, Imu3Gyr, Imu3Mag, Imu6, Imu9, Imu10, ImuQuat, Payload},
    types::Packable,
};
pub use ingester::Ingester;

/// INDTP standard payload enumeration.
#[derive(Debug)]
pub enum StandardPayload {
    /// Accelerometer only (for 3-axis sensor).
    Imu3Acc(Imu3Acc),
    /// Gyroscope only (for 3-axis sensor).
    Imu3Gyr(Imu3Gyr),
    /// Magnetometer only (for 3-axis sensor).
    Imu3Mag(Imu3Mag),
    /// Accelerometer + Gyroscope readings (for 6-axis sensor).
    Imu6(Imu6),
    /// Accelerometer + Gyroscope + Magnetometer readings
    /// (for 9-axis sensor).
    Imu9(Imu9),
    /// Accelerometer + Gyroscope + Magnetometer + Barometer readings
    /// (for 10-axis sensor).
    Imu10(Imu10),
    /// Attitude. Hamiltonian Quaternion (w, x, y, z).
    ImuQuat(ImuQuat),
}

impl StandardPayload {
    /// Convert payload to bytes.
    ///
    /// # Returns
    /// - Bytes representation of payload.
    #[must_use]
    pub fn to_bytes(&self) -> &[u8] {
        match self {
            Self::Imu3Acc(p) => p.to_bytes(),
            Self::Imu3Gyr(p) => p.to_bytes(),
            Self::Imu3Mag(p) => p.to_bytes(),
            Self::Imu6(p) => p.to_bytes(),
            Self::Imu9(p) => p.to_bytes(),
            Self::Imu10(p) => p.to_bytes(),
            Self::ImuQuat(p) => p.to_bytes(),
        }
    }

    /// Get payload type.
    ///
    /// # Returns
    /// - Payload type according to IDTP specification.
    #[must_use]
    pub const fn payload_type(&self) -> u8 {
        match self {
            Self::Imu3Acc(_) => Imu3Acc::TYPE_ID,
            Self::Imu3Gyr(_) => Imu3Gyr::TYPE_ID,
            Self::Imu3Mag(_) => Imu3Mag::TYPE_ID,
            Self::Imu6(_) => Imu6::TYPE_ID,
            Self::Imu9(_) => Imu9::TYPE_ID,
            Self::Imu10(_) => Imu10::TYPE_ID,
            Self::ImuQuat(_) => ImuQuat::TYPE_ID,
        }
    }

    /// Try to extract a standard payload from an IDTP frame.
    ///
    /// # Parameters
    /// - `frame` - given IDTP frame to handle.
    ///
    /// # Returns
    /// - Standard payload - in case of success.
    /// - `None` - otherwise.
    pub fn try_from(payload: &[u8], payload_type: PayloadType) -> Option<Self> {
        match payload_type {
            PayloadType::Imu3Acc => {
                Imu3Acc::from_bytes(payload).ok().map(Self::Imu3Acc)
            }
            PayloadType::Imu3Gyr => {
                Imu3Gyr::from_bytes(payload).ok().map(Self::Imu3Gyr)
            }
            PayloadType::Imu3Mag => {
                Imu3Mag::from_bytes(payload).ok().map(Self::Imu3Mag)
            }
            PayloadType::Imu6 => Imu6::from_bytes(payload).ok().map(Self::Imu6),
            PayloadType::Imu9 => Imu9::from_bytes(payload).ok().map(Self::Imu9),
            PayloadType::Imu10 => {
                Imu10::from_bytes(payload).ok().map(Self::Imu10)
            }
            PayloadType::ImuQuat => {
                ImuQuat::from_bytes(payload).ok().map(Self::ImuQuat)
            }
            PayloadType::Reserved(_) => None,
        }
    }

    /// Get payload length from payload type.
    ///
    /// # Parameters
    /// - `payload_type` - given payload type to handle.
    ///
    /// # Returns
    /// - Payload length in bytes.
    #[must_use]
    pub fn len_from(payload_type: PayloadType) -> usize {
        match payload_type {
            PayloadType::Imu3Acc => Imu3Acc::len(),
            PayloadType::Imu3Gyr => Imu3Gyr::len(),
            PayloadType::Imu3Mag => Imu3Mag::len(),
            PayloadType::Imu6 => Imu6::len(),
            PayloadType::Imu9 => Imu9::len(),
            PayloadType::Imu10 => Imu10::len(),
            PayloadType::ImuQuat => ImuQuat::len(),
            PayloadType::Reserved(_) => 0,
        }
    }
}
