// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU communication handler.

use crate::{
    config,
    model::{AppEvent, FrameContext},
};
use anyhow::anyhow;
use nalgebra::{Quaternion, UnitQuaternion};
use std::ops::Range;
use tokio::{net::UdpSocket, sync::mpsc::Sender, time};
use tsilna_nav::{
    math::rng::Xorshift,
    protocol::idtp::{IDTP_FRAME_MAX_SIZE, IdtpFrame, IdtpHeader}
};

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
    #[must_use]
    pub const fn new(tx: Sender<AppEvent>) -> Self {
        Self { tx }
    }

    /// Start communication with IMU.
    ///
    /// # Errors
    /// - Error to sending data over MPSC.
    /// - IDTP frame parsing error.
    /// - Receiving data over Wi-Fi error.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        log::info!("Running Ingester");

        let pair = (config::UDP_IP_ADDR, config::UDP_PORT);
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

        let mut begin_interval =
            tokio::time::interval(time::Duration::from_secs(1));

        loop {
            tokio::select! {
                recv = socket.recv_from(&mut buffer) => {
                    let (len, _addr) = recv?;
                    let raw_frame = buffer.get(..len).ok_or_else(|| anyhow!("Buffer underflow"))?;

                    let mut is_valid = if IdtpFrame::validate(raw_frame, None).is_err() {
                        bad_packets += 1;
                        false
                    }
                    else {
                        true
                    };

                    let mut q: Option<UnitQuaternion<f32>> = None;

                    let frame = if let Ok(frame) = IdtpFrame::try_from(raw_frame) {
                        if let Some(header) = frame.header() {
                            // Checking correctness of the sequence number.
                            let sequence = header.sequence;

                            if sequence <= prev_sequence {
                                bad_packets += 1;
                                is_valid = false;
                            }

                            q = Some(
                                estimate_attitude(total_packets as u32, &frame)
                            );
                            prev_sequence = sequence;
                            Some(frame)
                        }
                        else {
                            bad_packets += 1;
                            None
                        }
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

                    let _ = self.tx.send(AppEvent::FrameReceived(Box::new(frame_ctx))).await;

                }
                _ = begin_interval.tick() => {
                    current_pps = packets_in_last_second;
                    packets_in_last_second = 0;
                }
            }
        }
    }
}

// TODO: move to separate module.
/// Estimate attitude based on IMU readings.
///
/// # Parameters
/// - `seed` - given seed for pseudo-random numbers generator.
/// - `frame` - given IDTP frame to handle.
///
/// # Returns
/// - Attitude in quaternion representation.
fn estimate_attitude(seed: u32, _frame: &IdtpFrame) -> UnitQuaternion<f32> {
    // TODO: estimate attitude using IMU readings.
    const Q_RANGE: Range<f32> = -1.0..1.0;

    let mut rng = Xorshift::new(seed);
    let q = Quaternion::new(
        rng.next_f32(Q_RANGE),
        rng.next_f32(Q_RANGE),
        rng.next_f32(Q_RANGE),
        rng.next_f32(Q_RANGE),
    );

    UnitQuaternion::from_quaternion(q)
}
