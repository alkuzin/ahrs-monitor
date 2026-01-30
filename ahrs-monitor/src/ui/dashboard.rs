// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Dashboard tab user interface implementation.

use crate::{
    model::{FrameContext, attitude::Attitude},
    ui::{TabViewer, utils::display_metric},
};
use eframe::epaint::Stroke;
use egui::{Align2, Color32, FontId, Pos2, Sense, vec2};
use nalgebra::{UnitQuaternion, Vector3};

/// Roll angle color.
const ROLL_COLOR: Color32 = Color32::LIGHT_RED;

/// Pitch angle color.
const PITCH_COLOR: Color32 = Color32::LIGHT_GREEN;

/// Yaw angle color.
const YAW_COLOR: Color32 = Color32::LIGHT_BLUE;

/// Dashboard tab handler.
#[derive(Debug, Default)]
pub struct DashboardTab;

impl TabViewer for DashboardTab {
    /// Get tab title.
    ///
    /// # Returns
    /// - Tab title string slice.
    fn title(&self) -> &'static str {
        "Dashboard"
    }

    /// Get tab icon.
    ///
    /// # Returns
    /// - Tab icon string slice.
    fn icon(&self) -> &'static str {
        "🗖"
    }

    /// Display tab.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    /// - `frame_ctx` - given current frame context to handle.
    fn ui(&mut self, ui: &mut egui::Ui, frame_ctx: &FrameContext) {
        if let Some(quaternion) = frame_ctx.quaternion {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Euler Angles History").strong());

                let plot_height = ui.available_height() * 0.45;

                ui.scope(|ui| {
                    ui.set_height(plot_height);
                    Self::display_attitude_plot(ui, &quaternion);

                    ui.painter().rect_filled(
                        ui.available_rect_before_wrap(),
                        4.0,
                        Color32::from_black_alpha(20)
                    );

                    ui.centered_and_justified(|ui| ui.label("ATTITUDE PLOT AREA"));
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.columns(2, |cols| {
                if let Some(col) = cols.get_mut(0) {
                    col.vertical(|ui| {
                        // Displaying attitude widget.
                        ui.group(|ui| {
                            ui.set_height(ui.available_height() * 0.90);
                            ui.set_width(ui.available_width());
                            ui.label(egui::RichText::new("Attitude"));
                            ui.separator();

                            display_attitude_widget(ui, &quaternion);
                        });

                    });
                }

                if let Some(col) = cols.get_mut(1) {
                    col.vertical(|ui| Self::display_attitude(ui, &quaternion));
                }
            });
        }
    }
}

impl DashboardTab {
    /// Display plot of attitude changing over the time.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    /// - `quaternion` - given quaternion to handle.
    fn display_attitude_plot(
        ui: &mut egui::Ui,
        quaternion: &UnitQuaternion<f32>,
    ) {
        // TODO:
    }

    /// Display attitude metrics.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    /// - `quaternion` - given quaternion to handle.
    fn display_attitude(ui: &mut egui::Ui, quaternion: &UnitQuaternion<f32>) {
        ui.group(|ui| {
            ui.set_height(ui.available_height() * 0.90);
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("EULER ANGLES").strong());
            });
            ui.separator();

            ui.group(|ui| {
                ui.vertical(|ui| {
                    let attitude = Attitude::from_quaternion(quaternion);

                    display_metric(
                        ui,
                        "Roll:",
                        &format!("{:.2}", attitude.roll),
                        Some("rad"),
                        Some(ROLL_COLOR),
                    );
                    display_metric(
                        ui,
                        "Pitch:",
                        &format!("{:.2}", attitude.pitch),
                        Some("rad"),
                        Some(PITCH_COLOR),
                    );
                    display_metric(
                        ui,
                        "Yaw:",
                        &format!("{:.2}", attitude.yaw),
                        Some("rad"),
                        Some(YAW_COLOR),
                    );
                });
            });

            // Displaying quaternion data.
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("QUATERNION").strong());
            });
            ui.separator();

            ui.group(|ui| {
                ui.vertical(|ui| {
                    display_metric(
                        ui,
                        "w:",
                        &format!("{:.6}", quaternion.w),
                        None,
                        Some(Color32::LIGHT_YELLOW),
                    );
                    display_metric(
                        ui,
                        "x:",
                        &format!("{:.6}", quaternion.i),
                        None,
                        Some(Color32::LIGHT_RED),
                    );
                    display_metric(
                        ui,
                        "y:",
                        &format!("{:.6}", quaternion.j),
                        None,
                        Some(Color32::LIGHT_GREEN),
                    );
                    display_metric(
                        ui,
                        "z:",
                        &format!("{:.6}", quaternion.k),
                        None,
                        Some(Color32::LIGHT_BLUE),
                    );
                });
            });
        });
    }
}

/// Cube vertices size.
const VERTICES_SIZE: f32 = 1.0;

/// Set of cube vertices.
const CUBE_VERTICES: [Vector3<f32>; 8] = [
    Vector3::new(-VERTICES_SIZE, -VERTICES_SIZE, -VERTICES_SIZE),
    Vector3::new(VERTICES_SIZE, -VERTICES_SIZE, -VERTICES_SIZE),
    Vector3::new(VERTICES_SIZE, VERTICES_SIZE, -VERTICES_SIZE),
    Vector3::new(-VERTICES_SIZE, VERTICES_SIZE, -VERTICES_SIZE),
    Vector3::new(-VERTICES_SIZE, -VERTICES_SIZE, VERTICES_SIZE),
    Vector3::new(VERTICES_SIZE, -VERTICES_SIZE, VERTICES_SIZE),
    Vector3::new(VERTICES_SIZE, VERTICES_SIZE, VERTICES_SIZE),
    Vector3::new(-VERTICES_SIZE, VERTICES_SIZE, VERTICES_SIZE),
];

/// Set of cube edges.
const CUBE_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

/// Display attitude widget.
///
/// # Parameters
/// - `ui` - given screen UI handler.
/// - `rotation` - given quaternion to handle.
fn display_attitude_widget(ui: &mut egui::Ui, rotation: &UnitQuaternion<f32>) {
    let (rect, _) = ui.allocate_at_least(ui.available_size(), Sense::hover());
    let center = rect.center();
    let scale = rect.width().min(rect.height()) * 0.2;

    let painter = ui.painter();

    let project = |v: Vector3<f32>| -> Pos2 {
        let rotated = rotation * v;
        // Negative Y value since in egui Y-axis points downwards.
        center + vec2(rotated.x, -rotated.y) * scale
    };

    // Rendering the cube.
    let cube_stroke = Stroke::new(1.0, Color32::from_gray(100));

    for &(i, j) in &CUBE_EDGES {
        if let Some(v_i) = CUBE_VERTICES.get(i)
            && let Some(v_j) = CUBE_VERTICES.get(j)
        {
            painter.line_segment([project(*v_i), project(*v_j)], cube_stroke);
        }
    }

    // Rendering the axes.
    let axes_scale = 1.5;

    let axes = [
        (Vector3::x() * axes_scale, ROLL_COLOR, "Roll (X)"),
        (Vector3::y() * axes_scale, PITCH_COLOR, "Pitch (Y)"),
        (Vector3::z() * axes_scale, YAW_COLOR, "Yaw (Z)"),
    ];

    for (axis_vec, color, label) in axes {
        let origin = project(Vector3::zeros());
        let end = project(axis_vec);
        let stroke = Stroke::new(2.0, color);

        painter.line_segment([origin, end], stroke);
        painter.text(
            end,
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(12.0),
            color,
        );
    }
}
