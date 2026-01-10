// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application handler related declarations.

use eframe::Frame;
use egui::Context;

/// Application handler.
#[derive(Default)]
pub struct App;

impl eframe::App for App {
    /// Repaint UI.
    ///
    /// # Parameters
    /// - `ctx` - given egui context to handle.
    /// - `frame` - given surroundings of the app.
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(&ctx, |_ui| {
        });
    }
}
