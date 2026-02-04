// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Simulator of sending IDTP frames over UDP.

use std::{ops::Range, time::Duration};
use tokio::{net::UdpSocket, time::Instant};
use tsilna_nav::{
    math::rng::Xorshift,
    protocol::idtp::{
        IdtpFrame, IdtpHeader, IdtpMode,
        payload::{Imu3Acc, Imu3Gyr, Imu3Mag, Imu9}
    },
};
use ahrs_monitor::config::{self, load_config};

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
pub fn generate_payload(state: u32) -> Imu9 {
    let mut rng = Xorshift::new(state);

    Imu9 {
        acc: Imu3Acc {
            acc_x: rng.next_f32(RNG_ACC_RANGE),
            acc_y: rng.next_f32(RNG_ACC_RANGE),
            acc_z: rng.next_f32(RNG_ACC_RANGE),
        },
        gyr: Imu3Gyr {
            gyr_x: rng.next_f32(RNG_GYR_RANGE),
            gyr_y: rng.next_f32(RNG_GYR_RANGE),
            gyr_z: rng.next_f32(RNG_GYR_RANGE),
        },
        mag: Imu3Mag {
            mag_x: rng.next_f32(RNG_MAG_RANGE),
            mag_y: rng.next_f32(RNG_MAG_RANGE),
            mag_z: rng.next_f32(RNG_MAG_RANGE),
        },
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app_config = load_config(config::CONFIG_FILE_PATH);

    let simulator_addr = format!(
        "{}:{}",
        app_config.net.simulator_ip_address,
        app_config.net.simulator_udp_port,
    );

    let target_addr = format!(
        "{}:{}", app_config.net.ip_address, app_config.net.udp_port
    );

    let socket = UdpSocket::bind(simulator_addr).await?;

    let mut sequence = 0u32;
    let mut buffer = vec![0u8; 64];
    let mut rng_state = 1;
    let delay_time = Duration::from_millis(5); // 5 ms (200 Hz).
    let start_time = Instant::now();

    loop {
        let payload = generate_payload(rng_state);

        // Setting the IDTP header.
        let mut header = IdtpHeader::new();
        header.mode = IdtpMode::Safety.into();
        header.device_id = 0xABCD;
        header.sequence = sequence;
        header.timestamp = start_time.elapsed().as_micros() as u32;

        // Setting the IDTP frame.
        let mut frame = IdtpFrame::new();
        frame.set_header(&header);
        let _ = frame.set_payload::<Imu9>(&payload);

        let frame_size = frame.size();
        let _ = frame.pack(&mut buffer[..frame_size], None);
        socket.send_to(&buffer, &target_addr).await?;

        sequence += 1;

        if sequence.is_multiple_of(100) {
            println!("Sequence: {sequence} Sent 100 packets");
        }

        tokio::time::sleep(delay_time).await;
        rng_state += 1;
    }
}
