// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Utils for IMU simulator.

use ahrs_monitor::{config::ImuMetrics, core::IdtpStandardPayload};
use std::ops::Range;
use tsilna_nav::{math::rng::Xorshift, protocol::idtp::payload::*};

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
) -> IdtpStandardPayload {
    let mut rng = Xorshift::new(state);

    // Generate random metric if enabled.
    let mut generate = |enabled: bool, range: Range<f32>| -> f32 {
        if enabled { rng.next_f32(range) } else { 0.0 }
    };

    match payload_type {
        PayloadType::Imu3Acc => IdtpStandardPayload::Imu3Acc(Imu3Acc {
            acc_x: generate(imu_metrics.acc, RNG_ACC_RANGE),
            acc_y: generate(imu_metrics.acc, RNG_ACC_RANGE),
            acc_z: generate(imu_metrics.acc, RNG_ACC_RANGE),
        }),
        PayloadType::Imu3Gyr => IdtpStandardPayload::Imu3Gyr(Imu3Gyr {
            gyr_x: generate(imu_metrics.gyr, RNG_GYR_RANGE),
            gyr_y: generate(imu_metrics.gyr, RNG_GYR_RANGE),
            gyr_z: generate(imu_metrics.gyr, RNG_GYR_RANGE),
        }),
        PayloadType::Imu3Mag => IdtpStandardPayload::Imu3Mag(Imu3Mag {
            mag_x: generate(imu_metrics.mag, RNG_MAG_RANGE),
            mag_y: generate(imu_metrics.mag, RNG_MAG_RANGE),
            mag_z: generate(imu_metrics.mag, RNG_MAG_RANGE),
        }),
        PayloadType::Imu6 => IdtpStandardPayload::Imu6(Imu6 {
            acc: Imu3Acc {
                acc_x: generate(imu_metrics.acc, RNG_ACC_RANGE),
                acc_y: generate(imu_metrics.acc, RNG_ACC_RANGE),
                acc_z: generate(imu_metrics.acc, RNG_ACC_RANGE),
            },
            gyr: Imu3Gyr {
                gyr_x: generate(imu_metrics.gyr, RNG_GYR_RANGE),
                gyr_y: generate(imu_metrics.gyr, RNG_GYR_RANGE),
                gyr_z: generate(imu_metrics.gyr, RNG_GYR_RANGE),
            },
        }),
        PayloadType::Imu9 => IdtpStandardPayload::Imu9(Imu9 {
            acc: Imu3Acc {
                acc_x: generate(imu_metrics.acc, RNG_ACC_RANGE),
                acc_y: generate(imu_metrics.acc, RNG_ACC_RANGE),
                acc_z: generate(imu_metrics.acc, RNG_ACC_RANGE),
            },
            gyr: Imu3Gyr {
                gyr_x: generate(imu_metrics.gyr, RNG_GYR_RANGE),
                gyr_y: generate(imu_metrics.gyr, RNG_GYR_RANGE),
                gyr_z: generate(imu_metrics.gyr, RNG_GYR_RANGE),
            },
            mag: Imu3Mag {
                mag_x: generate(imu_metrics.mag, RNG_MAG_RANGE),
                mag_y: generate(imu_metrics.mag, RNG_MAG_RANGE),
                mag_z: generate(imu_metrics.mag, RNG_MAG_RANGE),
            },
        }),
        PayloadType::Imu10 => IdtpStandardPayload::Imu10(Imu10 {
            acc: Imu3Acc {
                acc_x: generate(imu_metrics.acc, RNG_ACC_RANGE),
                acc_y: generate(imu_metrics.acc, RNG_ACC_RANGE),
                acc_z: generate(imu_metrics.acc, RNG_ACC_RANGE),
            },
            gyr: Imu3Gyr {
                gyr_x: generate(imu_metrics.gyr, RNG_GYR_RANGE),
                gyr_y: generate(imu_metrics.gyr, RNG_GYR_RANGE),
                gyr_z: generate(imu_metrics.gyr, RNG_GYR_RANGE),
            },
            mag: Imu3Mag {
                mag_x: generate(imu_metrics.mag, RNG_MAG_RANGE),
                mag_y: generate(imu_metrics.mag, RNG_MAG_RANGE),
                mag_z: generate(imu_metrics.mag, RNG_MAG_RANGE),
            },
            baro: generate(imu_metrics.baro, RNG_BARO_RANGE),
        }),
        PayloadType::ImuQuat => IdtpStandardPayload::ImuQuat(ImuQuat {
            w: generate(imu_metrics.quat, RNG_QUAT_RANGE),
            x: generate(imu_metrics.quat, RNG_QUAT_RANGE),
            y: generate(imu_metrics.quat, RNG_QUAT_RANGE),
            z: generate(imu_metrics.quat, RNG_QUAT_RANGE),
        }),
    }
}
