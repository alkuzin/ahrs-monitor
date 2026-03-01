// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

/// IMU data transmission simulation implementation.

use ahrs_monitor::{config::{self, AppConfig}};
use indtp::{Frame, Mode, engines::{SwCryptoEngine, SwIntegrityEngine}, types::CryptoKeys};
use tokio::{
    net::UdpSocket,
    time::{Duration, Instant},
};
use crate::utils::ImuSimulator;

/// IMU data transmission simulator.
pub struct Simulator {
    /// Application's configurations.
    cfg: AppConfig,
    /// Simulator's IP address.
    simulator_addr: String,
    /// AHRS Monitor's IP address.
    monitor_addr: String,
    /// Container for cryptographic keys.
    keys: CryptoKeys,
    /// IMU readings simulator.
    sim: ImuSimulator,
}

impl Simulator {
    /// Construct new simulator.
    ///
    /// # Parameters
    /// - `cfg` - given application's configurations to handle.
    ///
    /// # Returns
    /// - New simulator - in case of success.
    /// - `Err` - otherwise.
    pub fn new(cfg: AppConfig) -> anyhow::Result<Self> {
        let net_cfg = &cfg.net;

        let simulator_addr = format!(
            "{}:{}",
            net_cfg.simulator_ip_address.clone(),
            net_cfg.simulator_udp_port,
        );

        let monitor_addr =
            format!("{}:{}", net_cfg.ip_address.clone(), net_cfg.udp_port);

        Ok(Self {
            cfg,
            simulator_addr,
            monitor_addr,
            keys: CryptoKeys::new(*config::AES_KEY, *config::HMAC_KEY),
            sim: ImuSimulator::new(1234)?,
        })
    }

    /// Simulate IMU data transmission over UDP.
    ///
    /// # Returns
    /// - `Ok` - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - I/O errors.
    pub async fn simulate_udp_transmission(&mut self) -> anyhow::Result<()> {
        let socket = UdpSocket::bind(&self.simulator_addr).await?;

        log::info!("Listening on {} (UDP)", self.simulator_addr);
        log::info!("Sending to AHRS Monitor: {} (UDP)", self.monitor_addr);

        let mut buffer = vec![0u8; 256];

        let mut sequence = 0u16;
        let device_id = 0xAA;
        let payload_type = self.cfg.imu.payload_type();
        let mode = Mode::try_from(self.cfg.imu.protocol_mode)
            .unwrap_or(Mode::Lite);

        let mut frame = match mode {
            Mode::Lite => Frame::new_lite(&mut buffer, device_id, payload_type.as_u8()),
            Mode::Verified => Frame::new_verified(&mut buffer, device_id, payload_type.as_u8()),
            Mode::Trusted => Frame::new_trusted(&mut buffer, device_id, payload_type.as_u8()),
            Mode::Critical => Frame::new_critical(&mut buffer, device_id, payload_type.as_u8()),
        }?;

        let metrics = self.cfg.imu.metrics;
        let delay = 1000.0 / self.cfg.imu.sample_rate;
        let dt = delay / 1000.0;
        let delay_time = Duration::from_millis(delay as u64);
        let start_time = Instant::now();

        loop {
            let payload = self.sim.next_payload(dt, &payload_type, &metrics);

            frame.set_sequence(sequence);
            let timestamp = start_time.elapsed().as_micros() as u32;
            frame.push_single_sample(timestamp, payload.to_bytes())?;

            let _ = frame.pack::<SwIntegrityEngine, SwCryptoEngine>(Some(&self.keys))?;
            let raw_frame = frame.frame()?;
            socket.send_to(raw_frame, &self.monitor_addr).await?;

            sequence += 1;

            if sequence.is_multiple_of(1000) {
                println!("Sequence: {sequence} Sent 1000 packets over UDP");
            }

            tokio::time::sleep(delay_time).await;
        }
    }
}
