// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Utils for IMU simulator.

use ahrs_monitor::{config::ImuMetrics, core::StandardPayload};
use indtp::payload::{Imu10, Imu3Acc, Imu3Gyr, Imu3Mag, Imu6, Imu9, ImuQuat, PayloadType};
use rand::prelude::*;
use rand_distr::{Normal, Distribution};

/// Earth's standard gravity in meters per second squared.
const GRAVITY: f32 = 9.80665;

/// IMU readings simulator.
pub struct ImuSimulator {
    /// Internal clock for periodic wave generation.
    time: f32,
    /// Current simulated orientation (normalized).
    quat: [f32; 4],
    /// Current angular velocity (rad/s).
    gyr: [f32; 3],
    /// Last barometer reading (Pa).
    last_baro: f32,
    /// Pseudo-random number generator.
    rng: StdRng,
    /// Noise generator (Normal distribution).
    noise_gen: Normal<f64>,
}

impl ImuSimulator {
    /// Construct new IMU readings simulator.
    ///
    /// # Parameters
    /// - `seed` - given pseudo-random number generator seed to handle.
    ///
    /// # Returns
    /// - New IMU readings simulator - in case of success.
    /// - `Err` - otherwise.
    pub fn new(seed: u64) -> anyhow::Result<Self> {
        Ok(Self {
            time: 0.0,
            quat: [1.0, 0.0, 0.0, 0.0],
            gyr: [0.0, 0.0, 0.0],
            last_baro: 101325.0,
            rng: StdRng::seed_from_u64(seed),
            noise_gen: Normal::new(0.0, 0.02)?,
        })
    }

    /// Generate noise.
    ///
    /// # Returns
    /// - Generated noise.
    #[inline]
    fn next_f32(&mut self) -> f32 {
        self.noise_gen.sample(&mut self.rng) as f32
    }

    /// Generate the next set of IMU readings.
    ///
    /// # Parameters
    /// - `dt` - given delta time in seconds.
    /// - `payload_type` - given standard payload type to handle.
    /// - `metrics` - given IMU metrics to handle.
    ///
    /// # Returns
    /// - Next generated set of IMU readings.
    pub fn next_payload(&mut self, dt: f32, payload_type: &PayloadType, metrics: &ImuMetrics) -> StandardPayload {
        self.time += dt;

        // Generating gyroscope readings.
        for i in 0..3 {
            let t = self.time;
            let swing = (t * (1.2 + i as f32)).sin() * 100.0;
            let jitter = (t * 25.0).sin() * 1.15;
            self.gyr[i] = if metrics.gyr { swing + jitter } else { 0.0 };
        }

        // Generating quaternion.
        if metrics.quat {
            self.integrate_gyro(dt);
        }

        // Generating accelerometer readings.
        let (acc_x, acc_y, acc_z) = if metrics.acc {
            let gravity = self.get_gravity_vector();
            let mut jitter = || { (self.next_f32() - 0.5) * 2.3 };
            (
                gravity[0] + jitter(),
                gravity[1] + jitter(),
                gravity[2] + jitter()
            )
        } else {
            (0.0, 0.0, 0.0)
        };

        // Generating magnetometer readings.
        let (mag_x, mag_y, mag_z) = if metrics.mag {
            (25.0 + self.next_f32(), -15.0 + self.next_f32(), -40.0 + self.next_f32())
        } else {
            (0.0, 0.0, 0.0)
        };

        // Generating barometer readings.
        if metrics.baro {
            self.last_baro += (self.next_f32() - 0.5) * 2.0;
        }

        match payload_type {
            PayloadType::Imu3Acc => StandardPayload::Imu3Acc(Imu3Acc {
                acc_x: acc_x.into(), acc_y: acc_y.into(), acc_z: acc_z.into(),
            }),
            PayloadType::Imu3Gyr => StandardPayload::Imu3Gyr(Imu3Gyr {
                gyr_x: self.gyr[0].into(), gyr_y: self.gyr[1].into(), gyr_z: self.gyr[2].into(),
            }),
            PayloadType::Imu3Mag => StandardPayload::Imu3Mag(Imu3Mag {
                mag_x: mag_x.into(), mag_y: mag_y.into(), mag_z: mag_z.into(),
            }),
            PayloadType::Imu6 => StandardPayload::Imu6(Imu6 {
                acc: Imu3Acc { acc_x: acc_x.into(), acc_y: acc_y.into(), acc_z: acc_z.into() },
                gyr: Imu3Gyr { gyr_x: self.gyr[0].into(), gyr_y: self.gyr[1].into(), gyr_z: self.gyr[2].into() },
            }),
            PayloadType::Imu9 => StandardPayload::Imu9(Imu9 {
                acc: Imu3Acc { acc_x: acc_x.into(), acc_y: acc_y.into(), acc_z: acc_z.into() },
                gyr: Imu3Gyr { gyr_x: self.gyr[0].into(), gyr_y: self.gyr[1].into(), gyr_z: self.gyr[2].into() },
                mag: Imu3Mag { mag_x: mag_x.into(), mag_y: mag_y.into(), mag_z: mag_z.into() },
            }),
            PayloadType::Imu10 => StandardPayload::Imu10(Imu10 {
                acc: Imu3Acc { acc_x: acc_x.into(), acc_y: acc_y.into(), acc_z: acc_z.into() },
                gyr: Imu3Gyr { gyr_x: self.gyr[0].into(), gyr_y: self.gyr[1].into(), gyr_z: self.gyr[2].into() },
                mag: Imu3Mag { mag_x: mag_x.into(), mag_y: mag_y.into(), mag_z: mag_z.into() },
                baro: self.last_baro.into(),
            }),
            PayloadType::ImuQuat => StandardPayload::ImuQuat(ImuQuat {
                w: self.quat[0].into(),
                x: self.quat[1].into(),
                y: self.quat[2].into(),
                z: self.quat[3].into(),
            }),
            _ => unreachable!(),
        }
    }

    /// Rotate the internal gravity vector by the current orientation.
    ///
    /// # Returns
    /// - Direction of gravity in the body frame.
    #[inline]
    fn get_gravity_vector(&self) -> [f32; 3] {
        let [qw, qx, qy, qz] = self.quat;
        [
            2.0 * (qx * qz - qw * qy) * GRAVITY,
            2.0 * (qw * qx + qy * qz) * GRAVITY,
            (qw * qw - qx * qx - qy * qy + qz * qz) * GRAVITY,
        ]
    }

    /// Integrate gyroscope readings in order to get quaternion.
    ///
    /// # Parameters
    /// - `dt` - given delta time in seconds.
    fn integrate_gyro(&mut self, dt: f32) {
        let [w, x, y, z] = self.quat;
        let [gx, gy, gz] = self.gyr;

        let nw = w + 0.5 * dt * (-x * gx - y * gy - z * gz);
        let nx = x + 0.5 * dt * ( w * gx + y * gz - z * gy);
        let ny = y + 0.5 * dt * ( w * gy - x * gz + z * gx);
        let nz = z + 0.5 * dt * ( w * gz + x * gy - y * gx);

        let norm = (nw*nw + nx*nx + ny*ny + nz*nz).sqrt();
        self.quat = [nw / norm, nx / norm, ny / norm, nz / norm];
    }
}
