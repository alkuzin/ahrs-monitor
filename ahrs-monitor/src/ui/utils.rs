// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Utils for AHRS Monitor user interface.

use eframe::epaint::Color32;
use egui::RichText;
use egui_plot::{Corner, Legend, Line, Plot, PlotPoints};
use std::collections::VecDeque;

/// Display custom metric.
///
/// # Parameters
/// - `ui` - given screen UI handler.
/// - `name` - given metric name.
/// - `value` - given metric value.
/// - `unit` - given metric measurement unit.
/// - `color` - given metric value text color.
pub fn display_metric(
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
    /// - `start_idx` - given start index of specific metrics in history.
    /// - `labels` - given labels for 3 metrics to plot.
    #[allow(clippy::cast_precision_loss)]
    pub fn render_plot(
        &self,
        ui: &mut egui::Ui,
        id: &str,
        title: &str,
        start_idx: usize,
        labels: [&str; 3],
    ) {
        let plot_height = self.plot_height.unwrap_or(256.0);
        let x_range = POINTS as f64;

        let colors = [
            Color32::LIGHT_BLUE,
            Color32::LIGHT_RED,
            Color32::LIGHT_GREEN,
        ];

        ui.label(RichText::new(title).strong());

        Plot::new(id)
            .height(plot_height)
            .show_grid(true)
            .legend(Legend::default().position(Corner::RightTop))
            .include_x(0.0)
            .include_x(x_range)
            .show(ui, |plot_ui| {
                for i in 0..3 {
                    let history_idx = start_idx + i;
                    if let Some(sequence) = self.history.get(history_idx) {
                        let points: PlotPoints = sequence
                            .iter()
                            .enumerate()
                            .map(|(idx, &val)| [idx as f64, val])
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
