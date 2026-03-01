// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Utils for IMU simulator.

use ahrs_monitor::{config::ImuMetrics, core::StandardPayload};
use std::ops::Range;
use tsilna_nav::math::rng::Xorshift;
use indtp::payload::{Imu10, Imu3Acc, Imu3Gyr, Imu3Mag, Imu6, Imu9, ImuQuat, PayloadType};

/// Pseudo-random accelerometer readings range.
const RNG_ACC_RANGE: Range<f32> = -157.0..157.0; // +- 16g (m/s^2)

/// Pseudo-random gyroscope readings range.
const RNG_GYR_RANGE: Range<f32> = -35.0..35.0; // +-2000 DPS (rad/s).

/// Pseudo-random magnetometer readings range.
const RNG_MAG_RANGE: Range<f32> = -100.0..100.0; // (μT).

/// Pseudo-random barometer readings range.
const RNG_BARO_RANGE: Range<f32> = 90000.0..110000.0; // atm. (Pa)

/// Pseudo-random quaternion values range.
const RNG_QUAT_RANGE: Range<f32> = -1.0..1.0;

/// Generate payload with pseudo-random IMU sensors readings.
///
/// # Parameters
/// - `state` - given pseudo-random numbers generator initial state.
/// - `payload_type` - given payload type to handle.
/// - `imu_metrics` - given IMU metrics type to handle.
///
/// # Returns
/// - New generated payload.
pub fn generate_payload(
    state: u32,
    payload_type: &PayloadType,
    imu_metrics: &ImuMetrics,
) -> StandardPayload {
    let mut rng = Xorshift::new(state);

    // Generate random metric if enabled.
    let mut generate = |enabled: bool, range: Range<f32>| -> f32 {
        if enabled { rng.next_f32(range) } else { 0.0 }
    };

    match payload_type {
        PayloadType::Imu3Acc => StandardPayload::Imu3Acc(Imu3Acc {
            acc_x: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
            acc_y: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
            acc_z: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
        }),
        PayloadType::Imu3Gyr => StandardPayload::Imu3Gyr(Imu3Gyr {
            gyr_x: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
            gyr_y: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
            gyr_z: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
        }),
        PayloadType::Imu3Mag => StandardPayload::Imu3Mag(Imu3Mag {
            mag_x: generate(imu_metrics.mag, RNG_MAG_RANGE).into(),
            mag_y: generate(imu_metrics.mag, RNG_MAG_RANGE).into(),
            mag_z: generate(imu_metrics.mag, RNG_MAG_RANGE).into(),
        }),
        PayloadType::Imu6 => StandardPayload::Imu6(Imu6 {
            acc: Imu3Acc {
                acc_x: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
                acc_y: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
                acc_z: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
            },
            gyr: Imu3Gyr {
                gyr_x: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
                gyr_y: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
                gyr_z: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
            },
        }),
        PayloadType::Imu9 => StandardPayload::Imu9(Imu9 {
            acc: Imu3Acc {
                acc_x: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
                acc_y: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
                acc_z: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
            },
            gyr: Imu3Gyr {
                gyr_x: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
                gyr_y: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
                gyr_z: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
            },
            mag: Imu3Mag {
                mag_x: generate(imu_metrics.mag, RNG_MAG_RANGE).into(),
                mag_y: generate(imu_metrics.mag, RNG_MAG_RANGE).into(),
                mag_z: generate(imu_metrics.mag, RNG_MAG_RANGE).into(),
            },
        }),
        PayloadType::Imu10 => StandardPayload::Imu10(Imu10 {
            acc: Imu3Acc {
                acc_x: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
                acc_y: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
                acc_z: generate(imu_metrics.acc, RNG_ACC_RANGE).into(),
            },
            gyr: Imu3Gyr {
                gyr_x: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
                gyr_y: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
                gyr_z: generate(imu_metrics.gyr, RNG_GYR_RANGE).into(),
            },
            mag: Imu3Mag {
                mag_x: generate(imu_metrics.mag, RNG_MAG_RANGE).into(),
                mag_y: generate(imu_metrics.mag, RNG_MAG_RANGE).into(),
                mag_z: generate(imu_metrics.mag, RNG_MAG_RANGE).into(),
            },
            baro: generate(imu_metrics.baro, RNG_BARO_RANGE).into(),
        }),
        PayloadType::ImuQuat => StandardPayload::ImuQuat(ImuQuat {
            w: generate(imu_metrics.quat, RNG_QUAT_RANGE).into(),
            x: generate(imu_metrics.quat, RNG_QUAT_RANGE).into(),
            y: generate(imu_metrics.quat, RNG_QUAT_RANGE).into(),
            z: generate(imu_metrics.quat, RNG_QUAT_RANGE).into(),
        }),
        &PayloadType::Reserved(_) => unreachable!("Unexpected payload type"),
    }
}
