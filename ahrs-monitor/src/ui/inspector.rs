// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Packet inspector tab user interface implementation.

use crate::{
    model::{FrameContext, payload::Payload},
    ui::TabViewer,
};
use eframe::epaint::Color32;
use egui::{Layout, RichText};
use std::fmt::Write;
use tsilna_nav::protocol::idtp::{IdtpFrame, Mode};
use zerocopy::FromBytes;

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
    fn ui(&mut self, ui: &mut egui::Ui, frame_ctx: &FrameContext) {
        if let Some(frame) = &frame_ctx.frame {
            ui.horizontal_top(|ui| {
                let mut col_height: Option<f32> = None;

                let desired_size = egui::vec2(512.0, ui.available_height());
                ui.allocate_ui(desired_size, |ui| {
                    col_height = display_hex_dump_column(ui, frame_ctx, frame);
                });

                ui.add_space(8.0);

                let desired_size =
                    egui::vec2(ui.available_width(), ui.available_height());
                ui.allocate_ui(desired_size, |ui| {
                    if let Some(height) = col_height {
                        display_payload_column(ui, frame, height);
                    }
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
/// - Column height if frame and its contents are valid.
/// - `None` - otherwise.
fn display_hex_dump_column(
    ui: &mut egui::Ui,
    frame_ctx: &FrameContext,
    frame: &IdtpFrame,
) -> Option<f32> {
    if let Some(header) = frame.header() {
        let preamble = header.preamble.to_le_bytes();
        let preamble = std::str::from_utf8(&preamble).unwrap_or("Unknown");
        let timestamp = header.timestamp;
        let sequence = header.sequence;
        let device_id = header.device_id;
        let payload_size = header.payload_size;
        let version = header.version;
        let version_major = (version >> 4) & 0x0F;
        let version_minor = version & 0x0F;
        let version = format!("v{version_major}.{version_minor}");
        let mode = Mode::from(header.mode);
        let payload_type = header.payload_type;
        let crc = header.crc;

        let (mode_label, mode_color) = match mode {
            Mode::Lite => ("IDTP-L", Color32::RED),
            Mode::Safety => ("IDTP-S (CRC-32)", Color32::LIGHT_BLUE),
            Mode::Secure => ("IDTP-SEC (HMAC-SHA256)", Color32::GREEN),
            Mode::Unknown => ("Unknown", Color32::GRAY),
        };

        let (valid_label, valid_color) = if frame_ctx.is_valid {
            ("VALID", Color32::GREEN)
        } else {
            ("INVALID", Color32::RED)
        };

        let col1_rect =
            ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
                // Displaying hex dump of the frame bytes.
                ui.group(|ui| {
                    // ui.set_min_width(512.0);
                    display_hex_dump(ui, &frame_ctx.raw_frame);
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
                    display_metric(
                        ui,
                        "Timestamp:",
                        &timestamp,
                        Some("µs"),
                        None,
                    );
                    display_metric(ui, "Sequence:", &sequence, None, None);
                    display_metric(ui, "Device ID:", &device_id, None, None);
                    display_metric(
                        ui,
                        "Payload Size:",
                        &payload_size,
                        Some("bytes"),
                        None,
                    );
                    display_metric(
                        ui,
                        "Protocol Mode:",
                        &mode_label,
                        None,
                        Some(mode_color),
                    );
                    display_metric(ui, "Version:", &version, None, None);
                    display_metric(
                        ui,
                        "Payload Type:",
                        &payload_type,
                        None,
                        None,
                    );
                    display_metric(ui, "CRC:", &crc, None, None);
                });
            });

        return Some(col1_rect.response.rect.height());
    }

    None
}

/// Display payload metrics column user interface.
///
/// # Parameters
/// - `ui` - given screen UI handler.
/// - `frame` - given IDTP frame to handle.
/// - `col_height` - given hex dump column height in pixels.
fn display_payload_column(
    ui: &mut egui::Ui,
    frame: &IdtpFrame,
    col_height: f32,
) {
    if let Ok(payload_bytes) = frame.payload()
        && let Ok(payload) = Payload::read_from_prefix(payload_bytes)
    {
        let payload = payload.0;

        let acc_x = payload.acc_x;
        let acc_y = payload.acc_y;
        let acc_z = payload.acc_z;
        let gyr_x = payload.gyr_x;
        let gyr_y = payload.gyr_y;
        let gyr_z = payload.gyr_z;
        let mag_x = payload.mag_x;
        let mag_y = payload.mag_y;
        let mag_z = payload.mag_z;

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
                            display_metric(ui, "ACC X:", &acc_x, None, None);
                            display_metric(ui, "ACC Y:", &acc_y, None, None);
                            display_metric(ui, "ACC Z:", &acc_z, None, None);
                            display_metric(ui, "GYR X:", &gyr_x, None, None);
                            display_metric(ui, "GYR Y:", &gyr_y, None, None);
                            display_metric(ui, "GYR Z:", &gyr_z, None, None);
                            display_metric(ui, "MAG X:", &mag_x, None, None);
                            display_metric(ui, "MAG Y:", &mag_y, None, None);
                            display_metric(ui, "MAG Z:", &mag_z, None, None);
                        });
                    });
            });
        });
    }
}

/// Display custom metric.
///
/// # Parameters
/// - `ui` - given screen UI handler.
/// - `name` - given metric name.
/// - `value` - given metric value.
/// - `unit` - given metric measurement unit.
/// - `color` - given metric value text color.
fn display_metric(
    ui: &mut egui::Ui,
    name: &str,
    value: &impl ToString,
    unit: Option<&str>,
    color: Option<Color32>,
) {
    ui.horizontal(|ui| {
        let color = color.unwrap_or(Color32::WHITE);

        ui.label(name);
        ui.label(RichText::new(value.to_string()).color(color));

        if let Some(unit) = unit {
            ui.label(unit);
        }
    });

    ui.separator();
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
