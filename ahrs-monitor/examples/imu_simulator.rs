// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Simulator of sending IDTP frames over UDP.

use tokio::{net::UdpSocket, time::Instant};
use tsilna_nav::protocol::idtp;
use std::{time::Duration, mem};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:10001").await?;
    let target_addr = "127.0.0.1:10000";

    let mut sequence = 0u32;
    let start_time = Instant::now();

    loop {
        let timestamp = start_time.elapsed().as_millis();
        let payload = generate_payload(0);

        let mut header = IdtpHeader::new();
        header.mode = Mode::Safety as u8;
        header.device_id = 0xABCD;
        header.sequence = sequence;
        header.timestamp = start_time.elapsed().as_micros() as u32;

        let mut frame = IdtpFrame::new();
        frame.set_header(&header);
        let _ = frame.set_payload(&payload.as_bytes());

        let mut buffer = [0u8; 196];

        let _ = frame.pack(&mut buffer, None);
        socket.send_to(&buffer, target_addr).await?;

        sequence += 1;

        if sequence % 100 == 0 {
            println!("Sent 100 packets, current timestamp: {} ms", timestamp);
        }

        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

/// IDTP payload struct.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed)]
pub struct Payload {
    pub acc1_x: f32,
    /// The value of the projection of the acceleration vector (accelerometer 1)
    /// along the Y axis (m/s^2).
    pub acc1_y: f32,
    /// The value of the projection of the acceleration vector (accelerometer 1)
    /// along the Z axis (m/s^2).
    pub acc1_z: f32,
    /// Angular velocity (gyroscope 1) along the X axis (rad/s).
    pub gyr1_x: f32,
    /// Angular velocity (gyroscope 1) along the Y axis (rad/s).
    pub gyr1_y: f32,
    /// Angular velocity (gyroscope 1) along the Z axis (rad/s).
    pub gyr1_z: f32,
    /// (Magnetometer 1) value along the X axis (Gauss).
    pub mag1_x: f32,
    /// (Magnetometer 1) value along the Y axis (Gauss).
    pub mag1_y: f32,
    /// (Magnetometer 1) value along the Z axis (Gauss).
    pub mag1_z: f32,
    /// The value of the projection of the acceleration vector (accelerometer 2)
    /// along the X axis (m/s^2).
    pub acc2_x: f32,
    /// The value of the projection of the acceleration vector (accelerometer 2)
    /// along the Y axis (m/s^2).
    pub acc2_y: f32,
    /// The value of the projection of the acceleration vector (accelerometer 2)
    /// along the Z axis (m/s^2).
    pub acc2_z: f32,
    /// Angular velocity (gyroscope 2) along the X axis (rad/s).
    pub gyr2_x: f32,
    /// Angular velocity (gyroscope 2) along the Y axis (rad/s).
    pub gyr2_y: f32,
    /// Angular velocity (gyroscope 2) along the Z axis (rad/s).
    pub gyr2_z: f32,
    /// (Magnetometer 2) value along the X axis (Gauss).
    pub mag2_x: f32,
    /// (Magnetometer 2) value along the Y axis (Gauss).
    pub mag2_y: f32,
    /// (Magnetometer 2) value along the Z axis (Gauss).
    pub mag2_z: f32,
    /// The value of the projection of the acceleration vector (accelerometer 3)
    /// along the X axis (m/s^2).
    pub acc3_x: f32,
    /// The value of the projection of the acceleration vector (accelerometer 3)
    /// along the Y axis (m/s^2).
    pub acc3_y: f32,
    /// The value of the projection of the acceleration vector (accelerometer 3)
    /// along the Z axis (m/s^2).
    pub acc3_z: f32,
    /// Angular velocity (gyroscope 3) along the X axis (rad/s).
    pub gyr3_x: f32,
    /// Angular velocity (gyroscope 3) along the Y axis (rad/s).
    pub gyr3_y: f32,
    /// Angular velocity (gyroscope 3) along the Z axis (rad/s).
    pub gyr3_z: f32,
    /// (Magnetometer 3) value along the X axis (Gauss).
    pub mag3_x: f32,
    /// (Magnetometer 3) value along the Y axis (Gauss).
    pub mag3_y: f32,
    /// (Magnetometer 3) value along the Z axis (Gauss).
    pub mag3_z: f32,
    /// The value of the projection of the acceleration vector (accelerometer 4)
    /// along the X axis (m/s^2).
    pub acc4_x: f32,
    /// The value of the projection of the acceleration vector (accelerometer 4)
    /// along the Y axis (m/s^2).
    pub acc4_y: f32,
    /// The value of the projection of the acceleration vector (accelerometer 4)
    /// along the Z axis (m/s^2).
    pub acc4_z: f32,
    /// Angular velocity (gyroscope 4) along the X axis (rad/s).
    pub gyr4_x: f32,
    /// Angular velocity (gyroscope 4) along the Y axis (rad/s).
    pub gyr4_y: f32,
    /// Angular velocity (gyroscope 4) along the Z axis (rad/s).
    pub gyr4_z: f32,
    /// (Magnetometer 4) value along the X axis (Gauss).
    pub mag4_x: f32,
    /// (Magnetometer 4) value along the Y axis (Gauss).
    pub mag4_y: f32,
    /// (Magnetometer 4) value along the Z axis (Gauss).
    pub mag4_z: f32,
    /// Pressure value (barometer 1) (Pascal).
    pub baro1: f32,
    /// Pressure value (barometer 2) (Pascal).
    pub baro2: f32,
    /// Pressure value (barometer 3) (Pascal).
    pub baro3: f32,
    /// Pressure value (barometer 4) (Pascal).
    pub baro4: f32,
}

/// Payload size in bytes.
pub const PAYLOAD_SIZE: usize = size_of::<Payload>();

impl Payload {
    /// Convert payload to bytes.
    ///
    /// # Returns
    /// - Payload byte array.
    pub fn as_bytes(&self) -> [u8; PAYLOAD_SIZE] {
        unsafe { mem::transmute::<Self, [u8; PAYLOAD_SIZE]>(*self) }
    }

    /// Convert a byte slice to a `Payload` struct.
    ///
    /// # Parameters
    /// - `bytes` - given bytes to convert.
    ///
    /// # Returns
    /// - Payload from bytes.
    pub fn from_bytes(bytes: &[u8; PAYLOAD_SIZE]) -> Self {
        unsafe { mem::transmute::<[u8; PAYLOAD_SIZE], Self>(*bytes) }
    }
}

use core::ops::Range;
use idtp::{IdtpFrame, IdtpHeader, Mode};
use rand::Rng;

/// Pseudo-random accelerometer readings range.
const RNG_ACC_RANGE: Range<f32> = -39.22..39.22; // +-4g.

/// Pseudo-random gyroscope readings range.
const RNG_GYR_RANGE: Range<f32> = -34.91..34.91; // 2000 DPS.

/// Pseudo-random magnetometer readings range.
const RNG_MAG_RANGE: Range<f32> = -8.0..8.0; // +-8 Gauss.

/// Pseudo-random barometer readings range.
const RNG_BARO_RANGE: Range<f32> = 95000.0..105000.0; // ~1 atm.

/// Generate payload with pseudo-random IMU sensors readings.
///
/// # Parameters
/// - `state` - given pseudo-random numbers generator initial state.
///
/// # Returns
/// - New generated payload.
pub fn generate_payload(state: u32) -> Payload {
    let mut rng = rand::rng();
    let mut payload = Payload::default();

    payload.acc1_x = rng.random_range(RNG_ACC_RANGE);
    payload.acc1_y = rng.random_range(RNG_ACC_RANGE);
    payload.acc1_z = rng.random_range(RNG_ACC_RANGE);
    payload.gyr1_x = rng.random_range(RNG_GYR_RANGE);
    payload.gyr1_y = rng.random_range(RNG_GYR_RANGE);
    payload.gyr1_z = rng.random_range(RNG_GYR_RANGE);
    payload.mag1_x = rng.random_range(RNG_MAG_RANGE);
    payload.mag1_y = rng.random_range(RNG_MAG_RANGE);
    payload.mag1_z = rng.random_range(RNG_MAG_RANGE);

    payload.acc2_x = rng.random_range(RNG_ACC_RANGE);
    payload.acc2_y = rng.random_range(RNG_ACC_RANGE);
    payload.acc2_z = rng.random_range(RNG_ACC_RANGE);
    payload.gyr2_x = rng.random_range(RNG_GYR_RANGE);
    payload.gyr2_y = rng.random_range(RNG_GYR_RANGE);
    payload.gyr2_z = rng.random_range(RNG_GYR_RANGE);
    payload.mag2_x = rng.random_range(RNG_MAG_RANGE);
    payload.mag2_y = rng.random_range(RNG_MAG_RANGE);
    payload.mag2_z = rng.random_range(RNG_MAG_RANGE);

    payload.acc3_x = rng.random_range(RNG_ACC_RANGE);
    payload.acc3_y = rng.random_range(RNG_ACC_RANGE);
    payload.acc3_z = rng.random_range(RNG_ACC_RANGE);
    payload.gyr3_x = rng.random_range(RNG_GYR_RANGE);
    payload.gyr3_y = rng.random_range(RNG_GYR_RANGE);
    payload.gyr3_z = rng.random_range(RNG_GYR_RANGE);
    payload.mag3_x = rng.random_range(RNG_MAG_RANGE);
    payload.mag3_y = rng.random_range(RNG_MAG_RANGE);
    payload.mag3_z = rng.random_range(RNG_MAG_RANGE);

    payload.acc4_x = rng.random_range(RNG_ACC_RANGE);
    payload.acc4_y = rng.random_range(RNG_ACC_RANGE);
    payload.acc4_z = rng.random_range(RNG_ACC_RANGE);
    payload.gyr4_x = rng.random_range(RNG_GYR_RANGE);
    payload.gyr4_y = rng.random_range(RNG_GYR_RANGE);
    payload.gyr4_z = rng.random_range(RNG_GYR_RANGE);
    payload.mag4_x = rng.random_range(RNG_MAG_RANGE);
    payload.mag4_y = rng.random_range(RNG_MAG_RANGE);
    payload.mag4_z = rng.random_range(RNG_MAG_RANGE);

    payload.baro1 = rng.random_range(RNG_BARO_RANGE);
    payload.baro2 = rng.random_range(RNG_BARO_RANGE);
    payload.baro3 = rng.random_range(RNG_BARO_RANGE);
    payload.baro4 = rng.random_range(RNG_BARO_RANGE);

    payload
}
