// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! AHRS Monitor core main module.
//! The core responsible for handling IDTP frames.

pub mod attitude;
mod ingester;

pub use ingester::Ingester;
use tsilna_nav::protocol::idtp::{
    IdtpFrame,
    payload::{
        IdtpPayload, Imu3Acc, Imu3Gyr, Imu3Mag, Imu6, Imu9, Imu10, ImuQuat,
    },
};

/// IDTP standard payload enumeration.
pub enum IdtpStandardPayload {
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

impl IdtpStandardPayload {
    /// Convert payload to bytes.
    ///
    /// # Returns
    /// - Bytes representation of payload.
    pub fn to_bytes(&self) -> &[u8] {
        match self {
            IdtpStandardPayload::Imu3Acc(p) => p.to_bytes(),
            IdtpStandardPayload::Imu3Gyr(p) => p.to_bytes(),
            IdtpStandardPayload::Imu3Mag(p) => p.to_bytes(),
            IdtpStandardPayload::Imu6(p) => p.to_bytes(),
            IdtpStandardPayload::Imu9(p) => p.to_bytes(),
            IdtpStandardPayload::Imu10(p) => p.to_bytes(),
            IdtpStandardPayload::ImuQuat(p) => p.to_bytes(),
        }
    }

    /// Get payload type.
    ///
    /// # Returns
    /// - Payload type according to IDTP specification.
    pub fn payload_type(&self) -> u8 {
        match self {
            IdtpStandardPayload::Imu3Acc(_) => Imu3Acc::TYPE_ID,
            IdtpStandardPayload::Imu3Gyr(_) => Imu3Gyr::TYPE_ID,
            IdtpStandardPayload::Imu3Mag(_) => Imu3Mag::TYPE_ID,
            IdtpStandardPayload::Imu6(_) => Imu6::TYPE_ID,
            IdtpStandardPayload::Imu9(_) => Imu9::TYPE_ID,
            IdtpStandardPayload::Imu10(_) => Imu10::TYPE_ID,
            IdtpStandardPayload::ImuQuat(_) => ImuQuat::TYPE_ID,
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
    pub fn try_from_frame(frame: &IdtpFrame) -> Option<Self> {
        match frame.header().payload_type {
            Imu3Acc::TYPE_ID => {
                frame.payload::<Imu3Acc>().ok().map(Self::Imu3Acc)
            }
            Imu3Gyr::TYPE_ID => {
                frame.payload::<Imu3Gyr>().ok().map(Self::Imu3Gyr)
            }
            Imu3Mag::TYPE_ID => {
                frame.payload::<Imu3Mag>().ok().map(Self::Imu3Mag)
            }
            Imu6::TYPE_ID => frame.payload::<Imu6>().ok().map(Self::Imu6),
            Imu9::TYPE_ID => frame.payload::<Imu9>().ok().map(Self::Imu9),
            Imu10::TYPE_ID => frame.payload::<Imu10>().ok().map(Self::Imu10),
            ImuQuat::TYPE_ID => {
                frame.payload::<ImuQuat>().ok().map(Self::ImuQuat)
            }
            _ => None,
        }
    }
}
