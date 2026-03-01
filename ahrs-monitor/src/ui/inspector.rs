// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Packet inspector tab user interface implementation.

use crate::{
    config::AppConfig,
    model::FrameContext,
    ui::{
        TabViewer,
        utils::{display_metric, extract_readings},
    },
};
use eframe::epaint::Color32;
use egui::{Layout, RichText};
use std::fmt::Write;
use indtp::{Mode, payload::PayloadType};
use indtp::payload::Imu6;
use indtp::types::Packable;
use crate::core::StandardPayload;
use crate::model::FrameWrapper;

/// Packet inspector tab handler.
pub struct InspectorTab;

impl TabViewer for InspectorTab {
    /// Get tab title.
    ///
    /// # Returns
    /// - Tab title string slice.
    fn title(&self) -> &'static str {
        "Packet Inspector"
    }

    /// Get tab icon.
    ///
    /// # Returns
    /// - Tab icon string slice.
    fn icon(&self) -> &'static str {
        "🔍"
    }

    /// Display tab.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    /// - `frame_ctx` - given current frame context to handle.
    /// - `app_cfg` - given global config to handle.
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        frame_ctx: &FrameContext,
        app_cfg: &AppConfig,
    ) {
        if let Some(frame) = &frame_ctx.frame {
            ui.horizontal_top(|ui| {
                let mut col_height: f32 = 0.0;

                let desired_size = egui::vec2(512.0, ui.available_height());
                ui.allocate_ui(desired_size, |ui| {
                    if let Some(frame) = &frame_ctx.frame {
                        col_height = display_hex_dump_column(ui, &frame, frame_ctx.is_valid);
                    }
                });

                ui.add_space(8.0);

                let desired_size =
                    egui::vec2(ui.available_width(), ui.available_height());
                ui.allocate_ui(desired_size, |ui| {
                    display_payload_column(ui, frame, col_height, app_cfg);
                });
            });
        }
    }
}

/// Display hex dump column user interface.
///
/// # Parameters
/// - `ui` - given screen UI handler.
/// - `frame_ctx` - given frame context to handle.
/// - `frame` - given IDTP frame to handle.
///
/// # Returns
/// - Column height.
fn display_hex_dump_column(
    ui: &mut egui::Ui,
    frame: &FrameWrapper,
    is_valid: bool,
) -> f32 {
    let header = frame.header;

    let preamble = header.preamble.to_bytes();
    let preamble = std::str::from_utf8(&preamble).unwrap_or("Unknown");
    let version = header.version;
    let version_major = (version >> 4) & 0x0F;
    let version_minor = version & 0x0F;
    let flags = header.flags();
    let sequence = header.sequence;
    let device_id = header.device_id;
    let payload_size = header.payload_len;
    let version = format!("v{version_major}.{version_minor}");
    let payload_type = header.payload_type;
    let crc = header.crc;

    let (mode_label, mode_color) = {
        flags.mode().map_or(("Unknown", Color32::GRAY), |mode| {
            match mode {
                Mode::Lite => ("Lite", Color32::CYAN),
                Mode::Verified => ("Verified (CRC-32)", Color32::LIGHT_BLUE),
                Mode::Trusted => ("Trusted (CMAC-AES-128)", Color32::MAGENTA),
                Mode::Critical => ("Critical (HMAC-SHA256)", Color32::LIGHT_RED),
            }
        })
    };

    let (batch_label, batch_color) = if flags.is_batch() {
        ("Data aggregation mode is enabled", Color32::GREEN)
    } else {
        ("Data aggregation mode is disabled", Color32::GRAY)
    };

    let (encrypt_label, encrypt_color) = if flags.is_encrypted() {
        ("Payload is encrypted", Color32::GREEN)
    } else {
        ("Payload is not encrypted", Color32::YELLOW)
    };

    let (priority_label, priority_color) = if flags.is_high_priority() {
        ("Frame has high priority", Color32::YELLOW)
    } else {
        ("Frame has low priority", Color32::GRAY)
    };

    let (valid_label, valid_color) = if is_valid {
        ("VALID", Color32::GREEN)
    } else {
        ("INVALID", Color32::RED)
    };

    let col1_rect = ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
        // Displaying hex dump of the frame bytes.
        ui.group(|ui| {
            let mut raw_frame = Vec::with_capacity(frame.size);

            // 1. Get a reference to the payload if it exists
            let payload = frame.payload.as_ref();

            // 2. Define your fallback (default) data
            let default_payload = StandardPayload::Imu6(Imu6::default());

            // 3. Select which one to use without moving
            let payload = payload.unwrap_or(&default_payload);

            raw_frame.extend_from_slice(frame.header.to_bytes());
            raw_frame.extend_from_slice(payload.to_bytes());
            raw_frame.extend_from_slice(&frame.trailer);

            display_hex_dump(ui, &raw_frame);
        });

        ui.add_space(16.0);

        // Displaying IDTP header info.
        ui.group(|ui| {
            display_metric(
                ui,
                "Frame: is",
                &valid_label,
                None,
                Some(valid_color),
            );
            display_metric(ui, "Preamble:", &preamble, None, None);
            display_metric(ui, "Version:", &version, None, None);
            display_metric(ui, "Flags:", &flags.bits(), None, None);
            display_metric(
                ui,
                "Protocol Mode:",
                &mode_label,
                None,
                Some(mode_color),
            );
            display_metric(ui, "Batch:", &batch_label, None, Some(batch_color));
            display_metric(ui, "Encryption:", &encrypt_label, None, Some(encrypt_color));
            display_metric(ui, "Priority:", &priority_label, None, Some(priority_color));
            display_metric(ui, "Device ID:", &device_id, None, None);
            display_metric(ui, "Payload Type:", &payload_type, None, None);
            display_metric(ui, "Sequence:", &sequence, None, None);
            display_metric(
                ui,
                "Payload Length:",
                &payload_size,
                Some("bytes"),
                None,
            );
            display_metric(ui, "CRC:", &crc, None, None);
        });
    });

    col1_rect.response.rect.height()
}

/// Display payload metrics column user interface.
///
/// # Parameters
/// - `ui` - given screen UI handler.
/// - `frame` - given IDTP frame to handle.
/// - `col_height` - given hex dump column height in pixels.
/// - `app_cfg` - given global config to handle.
fn display_payload_column(
    ui: &mut egui::Ui,
    frame: &FrameWrapper,
    col_height: f32,
    app_cfg: &AppConfig,
) {
    let data = extract_readings(frame);
    let payload_type = app_cfg.imu.payload_type;

    let [
        metric0,
        metric1,
        metric2,
        metric3,
        metric4,
        metric5,
        metric6,
        metric7,
        metric8,
        metric9,
    ] = data;

    let imu_metrics = app_cfg.imu.metrics;
    let (acc_x, acc_y, acc_z) = (metric0, metric1, metric2);
    let (gyr_x, gyr_y, gyr_z) = {
        if payload_type == PayloadType::Imu3Gyr.as_u8() {
            (metric0, metric1, metric2)
        } else {
            (metric3, metric4, metric5)
        }
    };
    let (mag_x, mag_y, mag_z) = {
        if payload_type == PayloadType::Imu3Mag.as_u8() {
            (metric0, metric1, metric2)
        } else {
            (metric6, metric7, metric8)
        }
    };
    let baro = metric9;
    let (q_w, q_x, q_y, q_z) = (metric0, metric1, metric2, metric3);

    let acc_mu = Some("m/s^2");
    let gyr_mu = Some("rad/s");
    let mag_mu = Some("μT");
    let baro_mu = Some("Pa");

    ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
        ui.group(|ui| {
            let height = col_height.max(100.0);

            ui.set_width(ui.available_width());
            ui.set_max_height(height - 14.0);

            egui::ScrollArea::vertical()
                .id_salt("payload_metrics_scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Payload Metrics").strong());
                        ui.separator();

                        if imu_metrics.acc {
                            display_metric(
                                ui, "ACC X:", &acc_x, acc_mu, None,
                            );
                            display_metric(
                                ui, "ACC Y:", &acc_y, acc_mu, None,
                            );
                            display_metric(
                                ui, "ACC Z:", &acc_z, acc_mu, None,
                            );
                        }

                        if imu_metrics.gyr {
                            display_metric(
                                ui, "GYR X:", &gyr_x, gyr_mu, None,
                            );
                            display_metric(
                                ui, "GYR Y:", &gyr_y, gyr_mu, None,
                            );
                            display_metric(
                                ui, "GYR Z:", &gyr_z, gyr_mu, None,
                            );
                        }

                        if imu_metrics.mag {
                            display_metric(
                                ui, "MAG X:", &mag_x, mag_mu, None,
                            );
                            display_metric(
                                ui, "MAG Y:", &mag_y, mag_mu, None,
                            );
                            display_metric(
                                ui, "MAG Z:", &mag_z, mag_mu, None,
                            );
                        }

                        if imu_metrics.baro {
                            display_metric(
                                ui, "BARO:", &baro, baro_mu, None,
                            );
                        }

                        if imu_metrics.quat {
                            display_metric(ui, "QUAT W:", &q_w, None, None);
                            display_metric(ui, "QUAT X:", &q_x, None, None);
                            display_metric(ui, "QUAT Y:", &q_y, None, None);
                            display_metric(ui, "QUAT Z:", &q_z, None, None);
                        }
                    });
                });
        });
    });
}

/// Convert byte to ASCII.
///
/// # Parameters
/// - `ch` - given byte to convert.
///
/// # Returns
/// ASCII character to print.
#[inline]
const fn to_print(ch: u8) -> char {
    if ch.is_ascii_graphic() {
        ch as char
    } else {
        '.'
    }
}

/// Display hex dump of raw bytes.
///
/// # Parameters
/// - `ui` - given screen UI handler.
/// - `bytes` - given raw bytes to display
fn display_hex_dump(ui: &mut egui::Ui, bytes: &[u8]) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    let bytes_per_line = 16;

    for (row_idx, chunk) in bytes.chunks(bytes_per_line).enumerate() {
        let mut hex_line = String::with_capacity(64);
        let mut ascii_line = String::with_capacity(32);
        let offset_label = &format!("<{:08x}>  ", row_idx * bytes_per_line);

        // Hex data representation.
        for i in 0..bytes_per_line {
            if let Some(b) = chunk.get(i) {
                let _ = write!(hex_line, "{b:02x} ");
            } else {
                hex_line.push_str("   ");
            }

            if (i + 1) % 8 == 0 {
                hex_line.push(' ');
            }
        }

        // ASCII data representation.
        ascii_line.push('|');

        for &b in chunk {
            ascii_line.push(to_print(b));
        }

        ascii_line.push('|');

        ui.horizontal(|ui| {
            ui.label(RichText::new(offset_label).color(Color32::WHITE));
            ui.label(hex_line);
            ui.label(RichText::new(ascii_line).color(Color32::WHITE));
        });
    }
}
