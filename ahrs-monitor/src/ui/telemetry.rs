// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Telemetry tab user interface implementation.

use crate::{
    model::{FrameContext, payload::Payload},
    ui::TabViewer,
};
use egui::{Color32, RichText};
use egui_plot::{Corner, Legend, Line, Plot, PlotPoints};
use std::collections::VecDeque;
use zerocopy::FromBytes;

/// Packet inspector tab handler.
#[derive(Debug, Default)]
pub struct TelemetryTab {
    /// IMU readings history.
    history: [VecDeque<f64>; 9],
    /// Max number of entries in history.
    max_points: u32,
}

impl TelemetryTab {
    /// Construct new `TelemetryTab` object.
    ///
    /// # Parameters
    /// - `max_points` - given max number of entries in history.
    ///
    /// # Returns
    /// - New `TelemetryTab` object.
    #[must_use]
    pub fn new(max_points: u32) -> Self {
        let history = std::array::from_fn(|_| {
            VecDeque::with_capacity(max_points as usize)
        });

        Self {
            history,
            max_points,
        }
    }

    /// Append IMU readings to the points history.
    ///
    /// # Parameters
    /// - `frame_ctx` - given current frame context to handle.
    pub fn add_data(&mut self, frame_ctx: &FrameContext) {
        if let Some(frame) = frame_ctx.frame
            && let Ok(payload_bytes) = frame.payload()
            && let Ok(payload) = Payload::read_from_prefix(payload_bytes)
        {
            let payload = payload.0;

            let data: [f32; 9] = [
                payload.acc_x,
                payload.acc_y,
                payload.mag_z,
                payload.gyr_x,
                payload.gyr_y,
                payload.gyr_z,
                payload.mag_x,
                payload.mag_y,
                payload.mag_z,
            ];

            for (i, &val) in data.iter().enumerate() {
                if let Some(sequence) = self.history.get_mut(i) {
                    if sequence.len() >= self.max_points as usize {
                        sequence.pop_front();
                    }

                    sequence.push_back(f64::from(val));
                }
            }
        }
    }
}

impl TabViewer for TelemetryTab {
    /// Get tab title.
    ///
    /// # Returns
    /// - Tab title string slice.
    fn title(&self) -> &'static str {
        "Telemetry"
    }

    /// Get tab icon.
    ///
    /// # Returns
    /// - Tab icon string slice.
    fn icon(&self) -> &'static str {
        "📈"
    }

    /// Display tab.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    /// - `frame_ctx` - given current frame context to handle.
    fn ui(&mut self, ui: &mut egui::Ui, _frame_ctx: &FrameContext) {
        ui.vertical(|ui| {
            let plot_height = 200.0;
            let x_range = f64::from(self.max_points);

            let render_plot =
                |ui: &mut egui::Ui,
                 id: &str,
                 title: &str,
                 start_idx: usize,
                 labels: [&str; 3]| {
                    ui.label(RichText::new(title).strong());

                    Plot::new(id)
                        .height(plot_height)
                        .show_grid(true)
                        .legend(Legend::default().position(Corner::RightTop))
                        .include_x(0.0)
                        .include_x(x_range)
                        .show(ui, |plot_ui| {
                            let colors = [
                                Color32::LIGHT_BLUE,
                                Color32::LIGHT_RED,
                                Color32::LIGHT_GREEN,
                            ];

                            for i in 0..3 {
                                let history_idx = start_idx + i;
                                if let Some(sequence) =
                                    self.history.get(history_idx)
                                {
                                    #[allow(clippy::cast_precision_loss)]
                                    {
                                        let points: PlotPoints = sequence
                                            .iter()
                                            .enumerate()
                                            .map(|(idx, &val)| {
                                                [idx as f64, val]
                                            })
                                            .collect();

                                        if let Some(label) = labels.get(i)
                                            && let Some(color) = colors.get(i)
                                        {
                                            plot_ui.line(
                                                Line::new(*label, points)
                                                    .color(*color)
                                                    .width(0.8),
                                            );
                                        }
                                    }
                                }
                            }
                        });

                    ui.add_space(10.0);
                };

            render_plot(
                ui,
                "acc_p",
                "Accelerometer (m/s²)",
                0,
                ["Acc X", "Acc Y", "Acc Z"],
            );
            render_plot(
                ui,
                "gyr_p",
                "Gyroscope (deg/s)",
                3,
                ["Gyr X", "Gyr Y", "Gyr Z"],
            );
            render_plot(
                ui,
                "mag_p",
                "Magnetometer (µT)",
                6,
                ["Mag X", "Mag Y", "Mag Z"],
            );
        });
    }
}
