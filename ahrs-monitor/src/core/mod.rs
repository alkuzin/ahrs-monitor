// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! AHRS Monitor core main module.
//! The core responsible for handling IDTP frames.

use tsilna_nav::protocol::idtp::IDTP_FRAME_MAX_SIZE;
use tokio::net::UdpSocket;

/// Mediator between AHRS monitor and IMU.
pub struct Ingester;

impl Ingester {
    /// Construct new `Ingester` object.
    ///
    /// # Returns
    /// - New `Ingester` object.
    pub fn new() -> Self {
        Self {}
    }

    /// Start communication with IMU.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        log::info!("Running Ingester");

        // TODO: move IP address and port into separate file.
        let socket = UdpSocket::bind(("127.0.0.1", 10000)).await?;
        let mut buffer = [0u8; IDTP_FRAME_MAX_SIZE];

        log::info!("Listening for IDTP frames...");
        let mut packet_counter = 0;

        loop {
            let (len, addr) = socket.recv_from(&mut buffer).await?;

            if packet_counter % 100 == 0 {
                log::info!("({}) Received {} bytes from {}", packet_counter, len, addr);
                log::info!("Buffer: {:x?}", &buffer[..196]);
            }

            packet_counter += 1;
        }

        Ok(())
    }
}
