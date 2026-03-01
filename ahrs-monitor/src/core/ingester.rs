// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU communication handler.

use crate::{
    config::{self, AppConfig},
    core::attitude::{AttitudeEstimator, estimate_attitude},
    model::{AppEvent, FrameContext},
};
use tokio::{net::UdpSocket, sync::mpsc::Sender, time};
use tsilna_nav::math::Quat32;
use indtp::{MTU_SIZE, Frame};
use indtp::engines::{SwCryptoEngine, SwIntegrityEngine};
use indtp::payload::PayloadType;
use indtp::types::CryptoKeys;
use indtp::utils::is_sequence_correct;
use crate::core::StandardPayload;
use crate::model::FrameWrapper;

/// Mediator between AHRS monitor and IMU.
pub struct Ingester {
    /// MPSC sender handle.
    tx: Sender<AppEvent>,
    /// Application's configurations.
    cfg: AppConfig,
    /// Total number of invalid packets.
    bad_packets: usize,
    /// Previous frame sequence number.
    prev_sequence: Option<u16>,
    /// Last timestamp in microseconds.
    last_timestamp_us: Option<u32>,
    /// Orientation estimator.
    estimator: AttitudeEstimator,
    /// Container for cryptographic keys.
    keys: CryptoKeys,
}

impl Ingester {
    /// Construct new `Ingester` object.
    ///
    /// # Parameters
    /// - `tx` - given MPSC sender handle.
    /// - `cfg` - given application's configurations.
    ///
    /// # Returns
    /// - New `Ingester` object.
    #[must_use]
    pub fn new(tx: Sender<AppEvent>, cfg: AppConfig) -> Self {
        Self {
            tx,
            cfg,
            bad_packets: 0,
            prev_sequence: None,
            last_timestamp_us: None,
            estimator: AttitudeEstimator::new(),
            keys: CryptoKeys::new(*config::AES_KEY, *config::HMAC_KEY),
        }
    }

    /// Start communication with IMU.
    ///
    /// # Returns
    /// - `Ok` - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - Error to sending data over MPSC.
    /// - IDTP frame parsing error.
    /// - Receiving data over Wi-Fi error.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        log::info!("Running Ingester");

        let pair = (self.cfg.net.ip_address.clone(), self.cfg.net.udp_port);
        let bind_result = UdpSocket::bind(pair).await;

        // Sending UDP connection status.
        self.tx
            .send(AppEvent::UpdateConnectionStatus(bind_result.is_ok()))
            .await?;

        let socket = bind_result?;
        let mut buffer = [0u8; MTU_SIZE];

        log::info!("Listening for IDTP frames...");

        let mut total_packets: usize = 0;
        let mut packets_in_last_second: usize = 0;
        let mut current_pps: usize = 0;

        let mut begin_interval =
            tokio::time::interval(time::Duration::from_secs(1));

        loop {
            tokio::select! {
                recv = socket.recv_from(&mut buffer) => {
                    let (len, _addr) = recv?;

                    total_packets += 1;
                    packets_in_last_second += 1;

                    let mut frame_ctx = FrameContext::default();
                    let result = Frame::parse::<SwIntegrityEngine, SwCryptoEngine>(&mut buffer[..len], Some(&self.keys));

                    match result {
                        Ok(mut frame) => {
                            let header = frame.header();
                            let recv_seq = header.sequence.get();

                            if is_sequence_correct(recv_seq, self.prev_sequence) {
                                let payload_type = PayloadType::from(header.payload_type);

                                if frame.is_encrypted() {
                                    frame.decrypt::<SwCryptoEngine>(&self.keys)?;
                                }

                                if let Ok((timestamp, payload)) = frame.read_single_sample() {
                                    let payload = StandardPayload::try_from(&payload, payload_type);

                                    frame_ctx.quaternion = Some(self.estimate_attitude(timestamp, &payload));
                                    self.prev_sequence = Some(recv_seq);

                                    let frame_wrapper = FrameWrapper {
                                        header: frame.header().clone(),
                                        payload,
                                        trailer: frame.trailer()?.to_vec(),
                                        size: frame.size(),
                                        flags: frame.flags(),
                                    };

                                    frame_ctx.frame = Some(frame_wrapper);
                                    frame_ctx.timestamp = timestamp;
                                    frame_ctx.is_valid = true;
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Error: {e}");
                            self.bad_packets += 1;
                            frame_ctx.is_valid = false;
                        }
                    }

                    frame_ctx.total_packets = total_packets;
                    frame_ctx.bad_packets = self.bad_packets;
                    frame_ctx.pps = current_pps;

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

    /// Estimate IMU attitude.
    ///
    /// # Parameters
    /// - `frame` - given IDTP frame to handle. TODO:
    ///
    /// # Returns
    /// - Attitude in quaternion representation - in case of success.
    /// - `None` - otherwise.
    fn estimate_attitude(&mut self, timestamp: u32, payload: &Option<StandardPayload>) -> Quat32 {
        let default_dt = 1.0 / self.cfg.imu.sample_rate;
        let current_timestamp_us = timestamp;

        let dt = self.last_timestamp_us.map_or(default_dt, |prev_us| {
            let diff = if current_timestamp_us >= prev_us {
                current_timestamp_us - prev_us
            } else {
                (u32::MAX - prev_us).wrapping_add(current_timestamp_us)
            };

            #[allow(clippy::cast_precision_loss)]
            {
                (diff as f32 / 1_000_000.0).clamp(0.0001, 0.1)
            }
        });

        estimate_attitude(&mut self.estimator, payload, dt)
    }
}
