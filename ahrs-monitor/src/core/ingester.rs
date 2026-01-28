// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU communication handler.

use tsilna_nav::protocol::idtp::{IdtpFrame, IDTP_FRAME_MAX_SIZE};
use tokio::{net::UdpSocket, sync::mpsc::Sender, time};
use crate::{config, model::{AppEvent, FrameContext}};

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
        let mut prev_sequence: u32 = 0;
        let mut packets_in_last_second = 0;
        let mut current_pps = 0;

        let mut begin_interval = tokio::time::interval(time::Duration::from_secs(1));

        loop {
            tokio::select! {
                recv = socket.recv_from(&mut buffer) => {
                    let (len, _addr) = recv?;
                    let raw_frame = &buffer[..len];
                    let mut is_valid = true;

                    if IdtpFrame::validate(&raw_frame, None).is_err() {
                        bad_packets += 1;
                        is_valid = false;
                    }

                    let frame = match IdtpFrame::try_from(raw_frame) {
                        Ok(frame) => {
                            if let Some(header) = frame.header() {
                                // Checking correctness of the sequence number.
                                let sequence = header.sequence;

                                if sequence <= prev_sequence {
                                    bad_packets += 1;
                                    is_valid = false;
                                }

                                prev_sequence = sequence;
                                Some(frame)
                            }
                            else {
                                bad_packets += 1;
                                None
                            }
                        },
                        Err(_) => {
                            bad_packets += 1;
                            None
                        }
                    };

                    total_packets += 1;
                    packets_in_last_second += 1;

                    let frame_ctx = FrameContext {
                        frame,
                        total_packets,
                        bad_packets,
                        pps: current_pps,
                        is_valid,
                        raw_frame: raw_frame.to_vec(),
                    };

                    let _ = self.tx.send(AppEvent::FrameReceived(frame_ctx)).await;

                }
                _ = begin_interval.tick() => {
                    current_pps = packets_in_last_second;
                    packets_in_last_second = 0;
                }
            }
        }
    }
}
