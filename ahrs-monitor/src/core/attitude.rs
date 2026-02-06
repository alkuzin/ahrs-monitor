// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Attitude estimation related declarations.

use crate::core::IdtpStandardPayload;
use fusion_ahrs::Ahrs;
use tsilna_nav::{
    math::{
        Quat32,
        na::{Quaternion, Vector3},
    },
    protocol::idtp::IdtpFrame,
};

#[derive(Default)]
/// AHRS attitude estimator wrapper.
pub struct AttitudeEstimator {
    /// Complimentary filter handler.
    ahrs: Ahrs,
}

impl AttitudeEstimator {
    /// Construct new `AttitudeEstimator` object.
    ///
    /// # Returns
    /// - New `AttitudeEstimator` object.
    #[must_use]
    pub fn new() -> Self {
        Self { ahrs: Ahrs::new() }
    }

    /// Estimate attitude based on inertial sensors readings.
    ///
    /// # Parameters
    /// - `acc` - given vector of accelerometer readings in g (g).
    /// - `gyr` - given vector of gyroscope readings
    ///   in degrees per second (deg/s).
    /// - `dt` - given time step since last update in seconds (sec).
    ///
    /// # Returns
    /// - Estimated attitude in quaternion representation.
    pub fn estimate_imu(
        &mut self,
        acc: Vector3<f32>,
        gyr: Vector3<f32>,
        dt: f32,
    ) -> Quat32 {
        self.ahrs.update_no_magnetometer(gyr, acc, dt);
        self.ahrs.quaternion()
    }

    /// Estimate attitude based on inertial sensors readings.
    ///
    /// # Parameters
    /// - `acc` - given vector of accelerometer readings in g (g).
    /// - `gyr` - given vector of gyroscope readings
    ///   in degrees per second (deg/s).
    /// - `mag` - given vector of magnetometer readings in microteslas (µT).
    /// - `dt` - given time step since last update in seconds (sec).
    ///
    /// # Returns
    /// - Estimated attitude in quaternion representation.
    pub fn estimate_marg(
        &mut self,
        acc: Vector3<f32>,
        gyr: Vector3<f32>,
        mag: Vector3<f32>,
        dt: f32,
    ) -> Quat32 {
        self.ahrs.update(gyr, acc, mag, dt);
        self.ahrs.quaternion()
    }
}

// Earth's standard gravity in meters per second squared.
const GRAVITY: f32 = 9.80665;

// Conversion factor from radians to degrees.
const RAD_TO_DEG: f32 = 180.0 / std::f32::consts::PI;

/// Estimate attitude based on IMU readings.
///
/// # Parameters
/// - `estimator` - given attitude estimator.
/// - `frame` - given IDTP frame to handle.
/// - `dt` - given time step since last update in seconds (sec).
///
/// # Returns
/// - Attitude in quaternion representation.
pub fn estimate_attitude(
    estimator: &mut AttitudeEstimator,
    frame: &IdtpFrame,
    dt: f32,
) -> Quat32 {
    match IdtpStandardPayload::try_from_frame(frame) {
        // IMU 6-axis (Acc + Gyr).
        Some(IdtpStandardPayload::Imu6(p)) => {
            let (acc, gyr) = (p.acc, p.gyr);

            let acc = Vector3::new(
                acc.acc_x / GRAVITY,
                acc.acc_y / GRAVITY,
                acc.acc_z / GRAVITY,
            );

            let gyr = Vector3::new(
                gyr.gyr_x * RAD_TO_DEG,
                gyr.gyr_y * RAD_TO_DEG,
                gyr.gyr_z * RAD_TO_DEG,
            );

            estimator.estimate_imu(acc, gyr, dt)
        }

        // MARG 9-axis (Acc + Gyr + Mag).
        Some(IdtpStandardPayload::Imu9(p)) => {
            let (acc, gyr, mag) = (p.acc, p.gyr, p.mag);

            let acc = Vector3::new(
                acc.acc_x / GRAVITY,
                acc.acc_y / GRAVITY,
                acc.acc_z / GRAVITY,
            );

            let gyr = Vector3::new(
                gyr.gyr_x * RAD_TO_DEG,
                gyr.gyr_y * RAD_TO_DEG,
                gyr.gyr_z * RAD_TO_DEG,
            );

            // Mag is already in µT, so no conversion needed.
            let mag = Vector3::new(mag.mag_x, mag.mag_y, mag.mag_z);

            estimator.estimate_marg(acc, gyr, mag, dt)
        }

        Some(IdtpStandardPayload::ImuQuat(p)) => {
            Quat32::from_quaternion(Quaternion::new(p.w, p.x, p.y, p.z))
        }

        _ => estimator.ahrs.quaternion(),
    }
}
