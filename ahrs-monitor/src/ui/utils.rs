// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Utils for AHRS Monitor user interface.

use eframe::epaint::Color32;
use egui::RichText;

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
