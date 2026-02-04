// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! The core responsible for AHRS Monitor user interface.

use crate::{config::AppConfig, model::FrameContext};
pub use dashboard::DashboardTab;
pub use inspector::InspectorTab;
pub use telemetry::TelemetryTab;

mod dashboard;
mod inspector;
mod telemetry;
pub mod utils;

/// Application tabs enumeration.
pub enum AppTab {
    /// Tab for displaying 3D model.
    Dashboard(DashboardTab),
    /// Sensor readings plots.
    Telemetry(Box<TelemetryTab>),
    /// Tab for displaying raw packet inspector.
    Inspector(InspectorTab),
}

/// Application tab trait.
pub trait TabViewer {
    /// Get tab title.
    ///
    /// # Returns
    /// - Tab title string slice.
    fn title(&self) -> &str;

    /// Get tab icon.
    ///
    /// # Returns
    /// - Tab icon string slice.
    fn icon(&self) -> &str;

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
    );
}
