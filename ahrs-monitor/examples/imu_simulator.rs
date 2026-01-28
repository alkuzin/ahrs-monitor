// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Simulator of sending IDTP frames over UDP.

use ahrs_monitor::model::payload::Payload;
use std::{ops::Range, time::Duration};
use tokio::{net::UdpSocket, time::Instant};
use tsilna_nav::{
    math::rng::Xorshift,
    protocol::idtp::{IdtpFrame, IdtpHeader, Mode},
};
use zerocopy::IntoBytes;

/// Pseudo-random accelerometer readings range.
const RNG_ACC_RANGE: Range<f32> = -39.22..39.22; // +-4g.

/// Pseudo-random gyroscope readings range.
const RNG_GYR_RANGE: Range<f32> = -34.91..34.91; // 2000 DPS.

/// Pseudo-random magnetometer readings range.
const RNG_MAG_RANGE: Range<f32> = -8.0..8.0; // +-8 Gauss.

/// Generate payload with pseudo-random IMU sensors readings.
///
/// # Parameters
/// - `state` - given pseudo-random numbers generator initial state.
///
/// # Returns
/// - New generated payload.
pub fn generate_payload(state: u32) -> Payload {
    let mut rng = Xorshift::new(state);

    Payload {
        acc_x: rng.next_f32(RNG_ACC_RANGE),
        acc_y: rng.next_f32(RNG_ACC_RANGE),
        acc_z: rng.next_f32(RNG_ACC_RANGE),
        gyr_x: rng.next_f32(RNG_GYR_RANGE),
        gyr_y: rng.next_f32(RNG_GYR_RANGE),
        gyr_z: rng.next_f32(RNG_GYR_RANGE),
        mag_x: rng.next_f32(RNG_MAG_RANGE),
        mag_y: rng.next_f32(RNG_MAG_RANGE),
        mag_z: rng.next_f32(RNG_MAG_RANGE),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:10001").await?;
    let target_addr = "127.0.0.1:10000";

    let mut sequence = 0u32;
    let mut buffer = vec![0u8; 64];
    let mut rng_state = 1;
    let delay_time = Duration::from_millis(5); // 5 ms (200 Hz).
    let start_time = Instant::now();

    loop {
        let payload = generate_payload(rng_state);

        // Setting the IDTP header.
        let mut header = IdtpHeader::new();
        header.mode = Mode::Safety as u8;
        header.device_id = 0xABCD;
        header.sequence = sequence;
        header.timestamp = start_time.elapsed().as_micros() as u32;

        // Setting the IDTP frame.
        let mut frame = IdtpFrame::new();
        frame.set_header(&header);
        let _ = frame.set_payload(payload.as_bytes());

        let frame_size = frame.size().unwrap_or(0);
        let _ = frame.pack(&mut buffer[..frame_size], None);
        socket.send_to(&buffer, target_addr).await?;

        sequence += 1;

        if sequence.is_multiple_of(100) {
            println!("Sequence: {sequence} Sent 100 packets");
        }

        tokio::time::sleep(delay_time).await;
        rng_state += 1;
    }
}
