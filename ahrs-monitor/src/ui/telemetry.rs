// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Telemetry tab user interface implementation.

use crate::{
    config::AppConfig,
    model::FrameContext,
    ui::{
        TabViewer,
        utils::{Plotter, extract_readings},
    },
};
use eframe::epaint::Color32;
use tsilna_nav::protocol::idtp::{IdtpFrame, payload::PayloadType};

/// Number of metrics in history.
const HISTORY_ENTRIES: usize = 10;

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
    /// - `frame` - given IDTP frame to handle.
    /// - `payload_type` - given payload type to handle.
    pub fn add_data(&mut self, frame: &IdtpFrame, payload_type: &PayloadType) {
        let data = extract_readings(frame, payload_type);
        self.plotter.add_data(data);
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
    /// - `app_cfg` - given global config to handle.
    fn ui(&mut self, ui: &mut egui::Ui, _: &FrameContext, app_cfg: &AppConfig) {
        ui.vertical(|ui| {
            self.plotter.set_plot_height(Some(200.0));

            let imu_metrics = app_cfg.imu.metrics;
            let payload_type = app_cfg.imu.payload_type;

            let colors = [
                Color32::LIGHT_BLUE,
                Color32::LIGHT_RED,
                Color32::LIGHT_GREEN,
            ];

            let acc_indices = &[0, 1, 2];

            let gyr_indices = {
                if payload_type == PayloadType::Imu3Gyr.into() {
                    &[0, 1, 2]
                } else {
                    &[3, 4, 5]
                }
            };

            let mag_indices = {
                if payload_type == PayloadType::Imu3Mag.into() {
                    &[0, 1, 2]
                } else {
                    &[6, 7, 8]
                }
            };

            let baro_indices = &[9];

            egui::ScrollArea::vertical().show(ui, |ui| {
                if imu_metrics.acc {
                    self.plotter.render_plot(
                        ui,
                        "acc_p",
                        "Accelerometer (m/s²)",
                        acc_indices,
                        &["Acc X", "Acc Y", "Acc Z"],
                        &colors,
                    );
                }

                if imu_metrics.gyr {
                    self.plotter.render_plot(
                        ui,
                        "gyr_p",
                        "Gyroscope (rad/s)",
                        gyr_indices,
                        &["Gyr X", "Gyr Y", "Gyr Z"],
                        &colors,
                    );
                }

                if imu_metrics.mag {
                    self.plotter.render_plot(
                        ui,
                        "mag_p",
                        "Magnetometer (µT)",
                        mag_indices,
                        &["Mag X", "Mag Y", "Mag Z"],
                        &colors,
                    );
                }

                if imu_metrics.baro {
                    self.plotter.render_plot(
                        ui,
                        "baro_p",
                        "Pressure (Pa)",
                        baro_indices,
                        &["Baro"],
                        &[Color32::LIGHT_BLUE],
                    );
                }

                if imu_metrics.quat {
                    self.plotter.render_plot(
                        ui,
                        "quat_w_p",
                        "Attitude (Quaternion W)",
                        &[0],
                        &["W"],
                        &[Color32::WHITE],
                    );

                    self.plotter.render_plot(
                        ui,
                        "quat_x_p",
                        "Attitude (Quaternion X)",
                        &[1],
                        &["X"],
                        &[Color32::LIGHT_RED],
                    );

                    self.plotter.render_plot(
                        ui,
                        "quat_y_p",
                        "Attitude (Quaternion Y)",
                        &[2],
                        &["Y"],
                        &[Color32::LIGHT_GREEN],
                    );

                    self.plotter.render_plot(
                        ui,
                        "quat_z_p",
                        "Attitude (Quaternion Z)",
                        &[3],
                        &["Z"],
                        &[Color32::LIGHT_BLUE],
                    );
                }
            });
        });
    }
}
