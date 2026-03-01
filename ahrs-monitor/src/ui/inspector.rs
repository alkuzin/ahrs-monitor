// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Packet inspector tab user interface implementation.

use crate::ui::utils::display_metric_group;
use crate::{
    config::AppConfig,
    core::StandardPayload,
    model::{FrameContext, FrameWrapper},
    ui::{
        TabViewer,
        utils::{Metric, extract_readings},
    },
};
use eframe::epaint::Color32;
use egui::{Layout, RichText};
use indtp::{
    Mode,
    payload::{Imu6, PayloadType},
    types::Packable,
};
use std::fmt::Write;

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
                        col_height = display_hex_dump_column(
                            ui,
                            frame,
                            frame_ctx.is_valid,
                        );
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
    let payload_len = header.payload_len;
    let version = format!("v{version_major}.{version_minor}");
    let payload_type = header.payload_type;
    let crc = header.crc;

    let (mode_label, mode_color) = {
        flags
            .mode()
            .map_or(("Unknown", Some(Color32::GRAY)), |mode| match mode {
                Mode::Lite => ("Lite", Some(Color32::CYAN)),
                Mode::Verified => {
                    ("Verified (CRC-32)", Some(Color32::LIGHT_BLUE))
                }
                Mode::Trusted => {
                    ("Trusted (CMAC-AES-128)", Some(Color32::MAGENTA))
                }
                Mode::Critical => {
                    ("Critical (HMAC-SHA256)", Some(Color32::LIGHT_RED))
                }
            })
    };

    let flags_label = &format!("{:#02X}", &flags.bits());
    let device_id_label = &format!("{device_id:#02X}");
    let payload_type_label = &format!("{payload_type:#02X}");
    let sequence_label = &sequence.to_string();
    let payload_len = &payload_len.to_string();
    let crc_label = &format!("{crc:#04X}");

    let (batch_label, batch_color) = if flags.is_batch() {
        ("Data aggregation mode is enabled", Some(Color32::GREEN))
    } else {
        ("Data aggregation mode is disabled", Some(Color32::GRAY))
    };

    let (encrypt_label, encrypt_color) = if flags.is_encrypted() {
        ("Payload is encrypted", Some(Color32::GREEN))
    } else {
        ("Payload is not encrypted", Some(Color32::YELLOW))
    };

    let (priority_label, priority_color) = if flags.is_high_priority() {
        ("Frame has high priority", Some(Color32::YELLOW))
    } else {
        ("Frame has low priority", Some(Color32::GRAY))
    };

    let (valid_label, valid_color) = if is_valid {
        ("VALID", Some(Color32::GREEN))
    } else {
        ("INVALID", Some(Color32::RED))
    };

    let col1_rect = ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
        // Displaying hex dump of the frame bytes.
        ui.group(|ui| {
            let mut raw_frame = Vec::with_capacity(frame.size);
            let payload = frame.payload.as_ref();
            let default_payload = StandardPayload::Imu6(Imu6::default());
            let payload = payload.unwrap_or(&default_payload);

            raw_frame.extend_from_slice(frame.header.to_bytes());
            raw_frame.extend_from_slice(payload.to_bytes());
            raw_frame.extend_from_slice(&frame.trailer);

            display_hex_dump(ui, &raw_frame);
        });

        ui.add_space(16.0);

        let metrics_args: Vec<Metric> = vec![
            Metric::new("Frame: is", valid_label, None, valid_color),
            Metric::new("Preamble:", preamble, None, None),
            Metric::new("Version:", &version, None, None),
            Metric::new("Flags:", flags_label, None, None),
            Metric::new("Protocol Mode:", mode_label, None, mode_color),
            Metric::new("Batch:", batch_label, None, batch_color),
            Metric::new("Encryption:", encrypt_label, None, encrypt_color),
            Metric::new("Priority:", priority_label, None, priority_color),
            Metric::new("Device ID:", device_id_label, None, None),
            Metric::new("Payload Type:", payload_type_label, None, None),
            Metric::new("Sequence:", sequence_label, None, None),
            Metric::new("Payload Length:", payload_len, Some("bytes"), None),
            Metric::new("CRC:", crc_label, None, None),
        ];

        // Displaying protocol header info.
        ui.group(|ui| {
            for m in &metrics_args {
                m.display(ui);
            }
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
#[allow(clippy::indexing_slicing)]
fn display_payload_column(
    ui: &mut egui::Ui,
    frame: &FrameWrapper,
    col_height: f32,
    app_cfg: &AppConfig,
) {
    let data = extract_readings(frame);
    let pt = app_cfg.imu.payload_type;
    let imu = app_cfg.imu.metrics;

    ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.set_max_height(col_height.max(100.0) - 14.0);

            egui::ScrollArea::vertical().id_salt("payload_scroll").show(
                ui,
                |ui| {
                    ui.label(RichText::new("Payload Metrics").strong());
                    ui.separator();

                    // Logic-based grouping
                    if imu.acc {
                        display_metric_group(
                            ui,
                            "ACC",
                            &data[0..3],
                            Some("m/s^2"),
                        );
                    }

                    if imu.gyr {
                        let start = if pt == PayloadType::Imu3Gyr.as_u8() {
                            0
                        } else {
                            3
                        };
                        display_metric_group(
                            ui,
                            "GYR",
                            &data[start..start + 3],
                            Some("rad/s"),
                        );
                    }

                    if imu.mag {
                        let start = if pt == PayloadType::Imu3Mag.as_u8() {
                            0
                        } else {
                            6
                        };
                        display_metric_group(
                            ui,
                            "MAG",
                            &data[start..start + 3],
                            Some("μT"),
                        );
                    }

                    if imu.baro {
                        let val = format!("{:.6}", data[9]);
                        Metric::new("BARO:", &val, Some("Pa"), None)
                            .display(ui);
                    }

                    if imu.quat {
                        display_metric_group(ui, "QUAT", &data[0..4], None);
                    }
                },
            );
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
