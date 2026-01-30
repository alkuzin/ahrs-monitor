// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Telemetry tab user interface implementation.

use crate::{
    model::{FrameContext, payload::Payload},
    ui::{TabViewer, utils::Plotter},
};
use zerocopy::FromBytes;

/// Number of metrics in history.
const HISTORY_ENTRIES: usize = 9;

/// Max number of points in history per each metric.
const MAX_POINTS: usize = 1000;

/// Packet inspector tab handler.
#[derive(Debug, Default)]
pub struct TelemetryTab {
    /// Metrics plotter.
    plotter: Plotter<HISTORY_ENTRIES, MAX_POINTS>,
}

impl TelemetryTab {
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

            let data: [f32; HISTORY_ENTRIES] = [
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

            self.plotter.add_data(data);
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
            self.plotter.set_plot_height(Some(200.0));

            self.plotter.render_plot(
                ui,
                "acc_p",
                "Accelerometer (m/s²)",
                0,
                ["Acc X", "Acc Y", "Acc Z"],
            );
            self.plotter.render_plot(
                ui,
                "gyr_p",
                "Gyroscope (deg/s)",
                3,
                ["Gyr X", "Gyr Y", "Gyr Z"],
            );
            self.plotter.render_plot(
                ui,
                "mag_p",
                "Magnetometer (µT)",
                6,
                ["Mag X", "Mag Y", "Mag Z"],
            );
        });
    }
}
