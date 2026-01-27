// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU communication handler.

use anyhow::anyhow;
use tsilna_nav::protocol::idtp::{IdtpError, IdtpFrame, IDTP_FRAME_MAX_SIZE};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::{config, model::AppEvent};
use crate::model::FrameContext;

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
        let mut total_packets = 0;
        let mut bad_packets = 0;

        loop {
            let (len, _addr) = socket.recv_from(&mut buffer).await?;
            let raw_frame = &buffer[..len];

            if IdtpFrame::validate(&raw_frame, None).is_err() {
                bad_packets += 1;
            }

            let frame = match IdtpFrame::try_from(raw_frame) {
                Ok(frame) => {
                    Some(frame)
                },
                Err(_) => {
                    bad_packets += 1;
                    None
                }
            };

            total_packets += 1;

            let frame_ctx = FrameContext {
                frame,
                total_packets,
                bad_packets,
            };
            
            self.tx.send(AppEvent::FrameReceived(frame_ctx)).await?;
        }
    }
}
