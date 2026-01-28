// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! The core responsible for AHRS Monitor user interface.

use crate::model::FrameContext;
pub use inspector::InspectorTab;

mod inspector;

/// Application tabs enumeration.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum Tab {
    /// Tab for displaying 3D model.
    #[default]
    Dashboard,
    /// Sensor readings plots.
    Telemetry,
    /// Tab for displaying raw packet inspector.
    Inspector,
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
    fn ui(&mut self, ui: &mut egui::Ui, frame_ctx: &FrameContext);
}
