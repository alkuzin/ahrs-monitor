// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU communication handler.

use tsilna_nav::protocol::idtp::IDTP_FRAME_MAX_SIZE;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::{config, model::AppEvent};

/// Mediator between AHRS monitor and IMU.
pub struct Ingester {
    /// MPSC sender handle.
    tx: Sender<AppEvent>,
}

impl Ingester {
    /// Construct new `Ingester` object.
    /// 
    /// # Parameters
    /// - `tx` - given MPSC sender handle.
    /// 
    /// # Returns
    /// - New `Ingester` object.
    pub fn new(tx: Sender<AppEvent>) -> Self {
        Self { tx }
    }

    /// Start communication with IMU.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        log::info!("Running Ingester");

        let pair = (config::UDP_IP_ADDR, config::UDP_PORT);
        let bind_result = UdpSocket::bind(pair).await;

        // Sending UDP connection status.
        self.tx.send(
            AppEvent::UpdateConnectionStatus(bind_result.is_ok())
        ).await?;

        let socket = bind_result?;
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
    }
}
