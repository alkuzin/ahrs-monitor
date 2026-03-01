// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Attitude estimation related declarations.

use crate::core::StandardPayload;
use fusion_ahrs::Ahrs;
use tsilna_nav::{
    math::{
        Quat32,
        na::{Quaternion, Vector3},
    },
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

/// Estimate attitude based on IMU readings.
///
/// # Parameters
/// - `estimator` - given attitude estimator.
// - `frame` - given IDTP frame to handle. TODO:
/// - `dt` - given time step since last update in seconds (sec).
///
/// # Returns
/// - Attitude in quaternion representation.
pub fn estimate_attitude(
    estimator: &mut AttitudeEstimator,
    payload: &Option<StandardPayload>,
    dt: f32,
) -> Quat32 {
    match payload {
        // IMU 6-axis (Acc + Gyr).
        Some(StandardPayload::Imu6(p)) => {
            let (acc, gyr) = (p.acc, p.gyr);

            let acc = Vector3::new(
                acc.acc_x.get(),
                acc.acc_y.get(),
                acc.acc_z.get(),
            );

            let gyr = Vector3::new(
                gyr.gyr_x.get(),
                gyr.gyr_y.get(),
                gyr.gyr_z.get(),
            );

            estimator.estimate_imu(acc, gyr, dt)
        }

        // MARG 9-axis (Acc + Gyr + Mag).
        Some(StandardPayload::Imu9(p)) => {
            let (acc, gyr, mag) = (p.acc, p.gyr, p.mag);

            let acc = Vector3::new(
                acc.acc_x.get(),
                acc.acc_y.get(),
                acc.acc_z.get(),
            );

            let gyr = Vector3::new(
                gyr.gyr_x.get(),
                gyr.gyr_y.get(),
                gyr.gyr_z.get(),
            );

            let mag = Vector3::new(
                mag.mag_x.get(),
                mag.mag_y.get(),
                mag.mag_z.get()
            );

            estimator.estimate_marg(acc, gyr, mag, dt)
        }

        Some(StandardPayload::ImuQuat(p)) => {
            Quat32::from_quaternion(
                Quaternion::new(
                    p.w.get(), p.x.get(), p.y.get(), p.z.get()
                )
            )
        }

        _ => estimator.ahrs.quaternion(),
    }
}
