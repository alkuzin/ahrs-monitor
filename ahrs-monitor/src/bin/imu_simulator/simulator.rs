// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU simulator implementation.

use crate::utils::generate_payload;
use ahrs_monitor::config::AppConfig;
use tokio::{
    net::UdpSocket,
    time::{Duration, Instant},
};
use tsilna_nav::protocol::idtp::{IdtpFrame, IdtpHeader, IdtpMode};

/// IMU simulator struct.
pub struct ImuSimulator {
    /// Application's configurations.
    cfg: AppConfig,
    /// Simulator's IP address.
    simulator_addr: String,
    /// AHRS Monitor's IP address.
    monitor_addr: String,
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

        let mut sequence = 0u32;
        let mut buffer = vec![0u8; 64];
        let mut rng_state = 1;

        let mut header = IdtpHeader::new();
        header.mode = IdtpMode::Safety.into();
        header.device_id = 0xABCD;

        let mut frame = IdtpFrame::new();
        let payload_type = self.cfg.imu.payload_type();
        let imu_metrics = self.cfg.imu.metrics;

        let delay_time = Duration::from_millis(5); // 5 ms (200 Hz).
        let start_time = Instant::now();

        loop {
            let payload =
                generate_payload(rng_state, &payload_type, &imu_metrics);

            header.sequence = sequence;
            header.timestamp = start_time.elapsed().as_micros() as u32;

            frame.set_header(&header);
            let _ = frame
                .set_payload_raw(payload.to_bytes(), payload.payload_type());

            let frame_size = frame.size();
            let _ = frame.pack(&mut buffer[..frame_size], None);
            socket.send_to(&buffer, &self.monitor_addr).await?;

            sequence += 1;

            if sequence.is_multiple_of(1000) {
                println!("Sequence: {sequence} Sent 1000 packets over UDP");
            }

            tokio::time::sleep(delay_time).await;
            rng_state += 1;
        }
    }
}
