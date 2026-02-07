// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU communication handler.

use aes_gcm::{Aes128Gcm, Key, KeyInit, aead::{Aead, Nonce}};
use crate::{config::{self, AppConfig}, core::attitude::{AttitudeEstimator, estimate_attitude}, model::{AppEvent, FrameContext}};
use anyhow::anyhow;
use tokio::{net::UdpSocket, sync::mpsc::Sender, time};
use tsilna_nav::{
    math::Quat32,
    protocol::idtp::{IDTP_FRAME_MAX_SIZE, IdtpFrame, IdtpMode},
};

/// Mediator between AHRS monitor and IMU.
pub struct Ingester {
    /// MPSC sender handle.
    tx: Sender<AppEvent>,
    /// Application's configurations.
    cfg: AppConfig,
    /// Total number of invalid packets.
    bad_packets: usize,
    /// Previous frame sequence number.
    prev_sequence: u32,
    /// Last timestamp in microseconds.
    last_timestamp_us: Option<u32>,
    /// Orientation estimator.
    estimator: AttitudeEstimator,
    /// AES-GCM with a 128-bit key.
    aes_cipher: Option<Aes128Gcm>,
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
        let aes_cipher = if cfg.net.use_encryption {
            let aes_key = Key::<Aes128Gcm>::from_slice(config::AES_KEY);
            let aes_cipher = Aes128Gcm::new(aes_key);

            Some(aes_cipher)
        }
        else {
            None
        };

        Self {
            tx,
            cfg,
            bad_packets: 0,
            prev_sequence: 0,
            last_timestamp_us: None,
            estimator: AttitudeEstimator::new(),
            aes_cipher,
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
        let mut buffer = [0u8; IDTP_FRAME_MAX_SIZE];

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
                    let raw_frame = buffer.get(..len)
                        .ok_or_else(|| anyhow!("Buffer underflow"))?;

                    total_packets += 1;
                    packets_in_last_second += 1;

                    let mut is_valid = false;
                    let mut quaternion = None;
                    let mut frame_opt = None;

                    let raw_frame = if let Some(decrypted_frame) = self.decrypt_frame(raw_frame)
                    && let Some(frame) = Self::validate_frame(&decrypted_frame) {
                        quaternion = Some(self.estimate_attitude(&frame));
                        frame_opt = Some(frame);
                        is_valid = true;

                        let header = frame.header();
                        self.prev_sequence = header.sequence;
                        self.last_timestamp_us = Some(header.timestamp);
                        decrypted_frame.to_vec()
                    } else {
                        self.bad_packets += 1;
                        raw_frame.to_vec()
                    };

                    let frame_ctx = FrameContext {
                        frame: frame_opt,
                        total_packets,
                        bad_packets: self.bad_packets,
                        pps: current_pps,
                        is_valid,
                        raw_frame,
                        quaternion,
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

    /// Validate IDTP frame.
    ///
    /// # Parameters
    /// - `raw_frame` - given raw IDTP frame bytes to handle.
    ///
    /// # Returns
    /// - Correct IDTP frame - in case of success.
    /// - `None` - otherwise.
    fn validate_frame(raw_frame: &[u8]) -> Option<IdtpFrame> {
        let frame = IdtpFrame::try_from(raw_frame).ok()?;
        let header = frame.header();
        let mode = header.mode;

        let hmac_key = if mode == u8::from(IdtpMode::Secure) {
            Some(config::HMAC_KEY.as_ref())
        } else {
            None
        };

        if IdtpFrame::validate(raw_frame, hmac_key).is_err() {
            return None;
        }

        Some(frame)
    }

    /// Estimate IMU attitude.
    ///
    /// # Parameters
    /// - `frame` - given IDTP frame to handle.
    ///
    /// # Returns
    /// - Attitude in quaternion representation - in case of success.
    /// - `None` - otherwise.
    fn estimate_attitude(&mut self, frame: &IdtpFrame) -> Quat32 {
        let default_dt = 1.0 / self.cfg.imu.sample_rate;
        let header = frame.header();
        let current_timestamp_us = header.timestamp;

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

        estimate_attitude(&mut self.estimator, frame, dt)
    }

    /// Decrypt frame if encryption mode is enabled.
    ///
    /// # Parameters
    /// - `buffer` - given raw packet bytes to handle.
    ///
    /// # Returns
    /// - Decrypted frame - if encryption mode is enabled.
    /// - Same IDTP frame bytes - otherwise.
    /// - `None` - in case of failure.
    fn decrypt_frame(&self, buffer: &[u8]) -> Option<Vec<u8>> {
        if let Some(aes_cipher) = &self.aes_cipher {
            // Packet structure [Nonce (IV) 12 bytes][Encrypted IDTP frame].
            if buffer.len() < 12 {
                return None;
            }

            let (iv, ciphertext) = buffer.split_at(12);
            let iv = Nonce::<Aes128Gcm>::from_slice(iv);

            match aes_cipher.decrypt(iv, ciphertext) {
                Ok(plaintext) => Some(plaintext),
                Err(_) => None,
            }
        } else {
            Some(buffer.to_vec())
        }
    }
}
