// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Utils for AHRS Monitor user interface.

use crate::{core::StandardPayload, model::FrameWrapper};
use eframe::epaint::Color32;
use egui::RichText;
use egui_plot::{Corner, Legend, Line, Plot, PlotPoints};
use indtp::types::F32;
use std::collections::VecDeque;

/// Custom metric struct.
pub struct Metric<'a> {
    /// Metric name.
    pub name: &'a str,
    /// Metric value.
    pub value: &'a str,
    /// Measurement unit.
    pub unit: Option<&'a str>,
    /// Metric value text color.
    pub color: Option<Color32>,
}

impl<'a> Metric<'a> {
    /// Construct new custom metric.
    ///
    /// # Parameters
    /// - `name` - given metric name.
    /// - `value` - given metric value.
    /// - `unit` - given metric measurement unit.
    /// - `color` - given metric value text color.
    ///
    /// # Returns
    /// - New custom metric.
    #[must_use]
    #[inline]
    pub const fn new(
        name: &'a str,
        value: &'a str,
        unit: Option<&'a str>,
        color: Option<Color32>,
    ) -> Self {
        Self {
            name,
            value,
            unit,
            color,
        }
    }

    /// Display custom metric.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    pub fn display(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let color = self.color.unwrap_or(Color32::WHITE);

            ui.label(self.name);
            ui.label(RichText::new(self.value).color(color));

            if let Some(unit) = &self.unit {
                ui.label(*unit);
            }
        });

        ui.separator();
    }
}

/// Display group of metrics.
///
/// # Parameters
/// - `ui` - given screen UI handler.
/// - `label` - given metric group label.
/// - `values` - given metric values.
/// - `unit` - given metric measurement unit.
pub fn display_metric_group(
    ui: &mut egui::Ui,
    label: &str,
    values: &[f32],
    unit: Option<&str>,
) {
    let axes = ["X", "Y", "Z", "W"];

    for (i, &val) in values.iter().enumerate() {
        let name = format!("{} {}:", label, axes.get(i).unwrap_or(&"?"));
        let value_str = format!("{val:.6}");
        Metric::new(&name, &value_str, unit, None).display(ui);
    }
}

/// Metrics plotter struct.
#[derive(Debug)]
pub struct Plotter<const ENTRIES: usize, const POINTS: usize> {
    /// Metrics history.
    history: [VecDeque<f64>; ENTRIES],
    /// Plot height in pixels.
    plot_height: Option<f32>,
}

impl<const ENTRIES: usize, const POINTS: usize> Plotter<ENTRIES, POINTS> {
    /// Append data to the points history.
    ///
    /// # Parameters
    /// - `data` - given data to append.
    pub fn add_data(&mut self, data: [f32; ENTRIES]) {
        for (i, &val) in data.iter().enumerate() {
            if let Some(sequence) = self.history.get_mut(i) {
                if sequence.len() >= POINTS {
                    sequence.pop_front();
                }

                sequence.push_back(f64::from(val));
            }
        }
    }

    /// Get most recent value for each metric entry.
    ///
    /// # Returns
    /// - The most recent value for each metric entry - in case of success.
    /// - `None` - otherwise.
    #[must_use]
    pub fn last_data(&self) -> Option<[f64; ENTRIES]> {
        let mut last_values = [0.0; ENTRIES];

        if self.history.is_empty() {
            return None;
        }

        for (i, sequence) in self.history.iter().enumerate() {
            if let Some(&val) = sequence.back()
                && let Some(last) = last_values.get_mut(i)
            {
                *last = val;
            }
        }

        Some(last_values)
    }

    /// Set plot height.
    ///
    /// # Parameters
    /// - `height` - given plot height in pixels to set.
    #[inline]
    pub const fn set_plot_height(&mut self, height: Option<f32>) {
        self.plot_height = height;
    }

    /// Render metrics plot.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    /// - `id` - given plot identifier.
    /// - `title` - given title of the plot.
    /// - `indices` - given indices of specific metrics in history.
    /// - `labels` - given labels for each metric to plot.
    /// - `colors` - given colors for each metric to plot.
    #[allow(clippy::cast_precision_loss)]
    pub fn render_plot(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        title: &str,
        indices: &[usize],
        labels: &[&str],
        colors: &[Color32],
    ) {
        ui.label(RichText::new(title).strong());

        let plot_height = self.plot_height.unwrap_or(256.0);
        let x_range = POINTS as f64;

        let plot = Plot::new(id)
            .height(plot_height)
            .show_grid(true)
            .legend(Legend::default().position(Corner::RightTop))
            .include_x(0.0)
            .allow_double_click_reset(true)
            .include_x(x_range);

        plot.show(ui, |plot_ui| {
            for (i, history_idx) in indices.iter().enumerate() {
                if let Some(sequence) = self.history.get(*history_idx) {
                    let points: PlotPoints = sequence
                        .iter()
                        .enumerate()
                        .map(|(idx, &val)| [idx as f64, val])
                        .collect();

                    if let Some(label) = labels.get(i)
                        && let Some(color) = colors.get(i)
                    {
                        plot_ui.line(
                            Line::new(*label, points).color(*color).width(0.8),
                        );
                    }
                }
            }
        });

        ui.add_space(10.0);
    }
}

impl<const ENTRIES: usize, const POINTS: usize> Default
    for Plotter<ENTRIES, POINTS>
{
    /// Construct new `Plotter` object.
    ///
    /// # Returns
    /// - New `Plotter` object.
    fn default() -> Self {
        let history = std::array::from_fn(|_| VecDeque::with_capacity(POINTS));
        Self {
            history,
            plot_height: None,
        }
    }
}

/// Extract IMU reading from payload.
///
/// # Parameters
/// - `frame` - given IDTP frame to handle.
/// - `payload_type` - given payload type to handle.
#[must_use]
pub fn extract_readings(frame: &FrameWrapper) -> [f32; 10] {
    // Add padding to IMU data.
    #[allow(clippy::indexing_slicing)]
    let pad = |src: &[F32]| {
        let mut res: [F32; 10] = [0.0.into(); 10];
        let len = src.len().min(10);
        res[..len].copy_from_slice(&src[..len]);
        res
    };

    let data = frame.payload.as_ref().map_or_else(
        || [0.0.into(); 10],
        |payload| match payload {
            StandardPayload::Imu3Acc(p) => pad(&[p.acc_x, p.acc_y, p.acc_z]),
            StandardPayload::Imu3Gyr(p) => pad(&[p.gyr_x, p.gyr_y, p.gyr_z]),
            StandardPayload::Imu3Mag(p) => pad(&[p.mag_x, p.mag_y, p.mag_z]),
            StandardPayload::Imu6(p) => pad(&[
                p.acc.acc_x,
                p.acc.acc_y,
                p.acc.acc_z,
                p.gyr.gyr_x,
                p.gyr.gyr_y,
                p.gyr.gyr_z,
            ]),
            StandardPayload::Imu9(p) => pad(&[
                p.acc.acc_x,
                p.acc.acc_y,
                p.acc.acc_z,
                p.gyr.gyr_x,
                p.gyr.gyr_y,
                p.gyr.gyr_z,
                p.mag.mag_x,
                p.mag.mag_y,
                p.mag.mag_z,
            ]),
            StandardPayload::Imu10(p) => pad(&[
                p.acc.acc_x,
                p.acc.acc_y,
                p.acc.acc_z,
                p.gyr.gyr_x,
                p.gyr.gyr_y,
                p.gyr.gyr_z,
                p.mag.mag_x,
                p.mag.mag_y,
                p.mag.mag_z,
                p.baro,
            ]),
            StandardPayload::ImuQuat(p) => pad(&[p.w, p.x, p.y, p.x]),
        },
    );

    let data: [f32; 10] = data.map(F32::get);
    data
}
