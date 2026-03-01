// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU simulator implementation.

use indtp::{Frame, Mode};
use indtp::engines::{SwCryptoEngine, SwIntegrityEngine};
use indtp::types::CryptoKeys;
use crate::utils::generate_payload;
use ahrs_monitor::config::{self, AppConfig};
use tokio::{
    net::UdpSocket,
    time::{Duration, Instant},
};
use ahrs_monitor::core::StandardPayload;

/// IMU simulator struct.
pub struct ImuSimulator {
    /// Application's configurations.
    cfg: AppConfig,
    /// Simulator's IP address.
    simulator_addr: String,
    /// AHRS Monitor's IP address.
    monitor_addr: String,
    /// Container for cryptographic keys.
    keys: CryptoKeys,
}

impl ImuSimulator {
    /// Construct new `ImuSimulator` object.
    ///
    /// # Parameters
    /// - `cfg` - given application's configurations to handle.
    ///
    /// # Returns
    /// - New `ImuSimulator` object.
    pub fn new(cfg: AppConfig) -> Self {
        let net_cfg = &cfg.net;

        let simulator_addr = format!(
            "{}:{}",
            net_cfg.simulator_ip_address.clone(),
            net_cfg.simulator_udp_port,
        );

        let monitor_addr =
            format!("{}:{}", net_cfg.ip_address.clone(), net_cfg.udp_port);

        Self {
            cfg,
            simulator_addr,
            monitor_addr,
            keys: CryptoKeys::new(*config::AES_KEY, *config::HMAC_KEY),
        }
    }

    /// Simulate IMU data transmission over UDP.
    ///
    /// # Returns
    /// - `Ok` - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - I/O errors.
    pub async fn simulate_udp_transmission(&self) -> anyhow::Result<()> {
        let socket = UdpSocket::bind(&self.simulator_addr).await?;

        println!("Listening on {} (UDP)", self.simulator_addr);
        println!("Sending to AHRS Monitor: {} (UDP)", self.monitor_addr);

        let mut buffer = vec![0u8; 64];
        let mut sequence = 0u16;
        let mut rng_state = 1;

        let device_id = 0xAA;
        let payload_type = self.cfg.imu.payload_type();
        let payload_len = StandardPayload::len_from(payload_type);
        let mode = Mode::try_from(self.cfg.imu.protocol_mode).unwrap_or(Mode::Lite);

        let mut frame = match mode {
            Mode::Lite => Frame::new_lite(&mut buffer, device_id, payload_type.as_u8(), payload_len + 4),
            Mode::Verified => Frame::new_verified(&mut buffer, device_id, payload_type.as_u8(), payload_len + 4),
            Mode::Trusted => Frame::new_trusted(&mut buffer, device_id, payload_type.as_u8(), payload_len + 4),
            Mode::Critical => Frame::new_critical(&mut buffer, device_id, payload_type.as_u8(), payload_len + 4),
        }?;

        let imu_metrics = self.cfg.imu.metrics;
        let delay_time = Duration::from_millis(5); // 5 ms (200 Hz).
        let start_time = Instant::now();

        loop {
            let payload =
                generate_payload(rng_state, &payload_type, &imu_metrics);

            frame.set_sequence(sequence);
            let timestamp = start_time.elapsed().as_micros() as u32;

            frame.push_single_sample(timestamp, payload.to_bytes())?;
            let _ = frame.pack::<SwIntegrityEngine, SwCryptoEngine>(Some(&self.keys))?;

            socket.send_to(frame.frame().unwrap(), &self.monitor_addr).await?;
            sequence += 1;

            if sequence.is_multiple_of(1000) {
                println!("Sequence: {sequence} Sent 1000 packets over UDP");
            }

            tokio::time::sleep(delay_time).await;
            rng_state += 1;
        }
    }
}
