// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU communication handler.

use crate::core::attitude::AttitudeEstimator;
use crate::{
    config::NetConfig,
    core::attitude::estimate_attitude,
    model::{AppEvent, FrameContext},
};
use anyhow::anyhow;
use tokio::{net::UdpSocket, sync::mpsc::Sender, time};
use tsilna_nav::{
    math::Quat32,
    protocol::idtp::{IDTP_FRAME_MAX_SIZE, IdtpFrame},
};

/// Mediator between AHRS monitor and IMU.
pub struct Ingester {
    /// MPSC sender handle.
    tx: Sender<AppEvent>,
    /// Networks configurations.
    net_cfg: NetConfig,
}

impl Ingester {
    /// Construct new `Ingester` object.
    ///
    /// # Parameters
    /// - `tx` - given MPSC sender handle.
    /// - `cfg` - given networks related configurations.
    ///
    /// # Returns
    /// - New `Ingester` object.
    #[must_use]
    pub const fn new(tx: Sender<AppEvent>, cfg: NetConfig) -> Self {
        Self { tx, net_cfg: cfg }
    }

    /// Start communication with IMU.
    ///
    /// # Errors
    /// - Error to sending data over MPSC.
    /// - IDTP frame parsing error.
    /// - Receiving data over Wi-Fi error.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        log::info!("Running Ingester");

        let pair = (self.net_cfg.ip_address.clone(), self.net_cfg.udp_port);
        let bind_result = UdpSocket::bind(pair).await;

        // Sending UDP connection status.
        self.tx
            .send(AppEvent::UpdateConnectionStatus(bind_result.is_ok()))
            .await?;

        let socket = bind_result?;
        let mut buffer = [0u8; IDTP_FRAME_MAX_SIZE];

        log::info!("Listening for IDTP frames...");

        let mut total_packets: usize = 0;
        let mut bad_packets: usize = 0;
        let mut prev_sequence: u32 = 0;
        let mut packets_in_last_second: usize = 0;
        let mut current_pps: usize = 0;
        let mut last_timestamp_us: Option<u32> = None;

        let mut ahrs = AttitudeEstimator::new();

        let mut begin_interval =
            tokio::time::interval(time::Duration::from_secs(1));

        loop {
            tokio::select! {
                recv = socket.recv_from(&mut buffer) => {
                    let (len, _addr) = recv?;
                    let raw_frame = buffer.get(..len).ok_or_else(|| anyhow!("Buffer underflow"))?;

                    let is_valid = if IdtpFrame::validate(raw_frame, None).is_err() {
                        bad_packets += 1;
                        false
                    }
                    else {
                        true
                    };

                    let mut q: Option<Quat32> = None;

                    let frame = if let Ok(frame) = IdtpFrame::try_from(raw_frame) {
                        let header = frame.header();

                        let current_timestamp_us = header.timestamp;
                        let sequence = header.sequence;

                        // Calculating dt.
                        let dt = if let Some(prev_us) = last_timestamp_us {
                            let diff = if current_timestamp_us >= prev_us {
                                current_timestamp_us - prev_us
                            } else {
                                // Handling wrap-around.
                                (u32::MAX - prev_us).wrapping_add(current_timestamp_us)
                            };

                            (diff as f32 / 1_000_000.0).clamp(0.0001, 0.1)
                        } else {
                            0.005 // 200 Hz default for the first packet.
                        };

                        // Updating the timestamp if the sequence is actually moving forward.
                        if sequence > prev_sequence || last_timestamp_us.is_none() {
                            last_timestamp_us = Some(current_timestamp_us);
                            prev_sequence = sequence;
                        }

                        q = Some(
                            estimate_attitude(&mut ahrs, &frame, dt)
                        );

                        prev_sequence = sequence;
                        Some(frame)
                    }
                    else {
                        bad_packets += 1;
                        None
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
                        quaternion: q,
                    };

                    let _ = self.tx.send(
                        AppEvent::FrameReceived(Box::new(frame_ctx))
                    ).await;

                }
                _ = begin_interval.tick() => {
                    current_pps = packets_in_last_second;
                    packets_in_last_second = 0;
                }
            }
        }
    }
}
