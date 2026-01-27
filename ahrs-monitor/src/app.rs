// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application handler related declarations.

use eframe::Frame;
use egui::Context;

/// Application tabs enumeration.
#[derive(Debug, Default, PartialEq)]
enum Tab {
    /// Tab for displaying 3D model.
    #[default]
    Dashboard,
    /// Sensor readings plots.
    Telemetry,
    /// Tab for displaying raw packet inspector.
    Inspector,
}

/// Application handler.
#[derive(Default)]
pub struct App {
    /// Current selected tab.
    current_tab: Tab,
}

impl eframe::App for App {
    /// Repaint UI.
    ///
    /// # Parameters
    /// - `ctx` - given egui context to handle.
    /// - `frame` - given surroundings of the app.
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Dashboard, "Dashboard");
                ui.selectable_value(&mut self.current_tab, Tab::Telemetry, "Telemetry");
                ui.selectable_value(&mut self.current_tab, Tab::Inspector, "Packet Inspector");
            })
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            match self.current_tab {
                Tab::Inspector => self.view_packet_inspector_tab(ui),
                Tab::Dashboard => self.view_dashboard_tab(ui),
                Tab::Telemetry => self.view_telemetry_tab(ui),
            }
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Bottom panel");
            })
        });
    }
}

impl App {
    /// Display dashboard tab.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    fn view_dashboard_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Dashboard");
    }

    /// Display telemetry tab.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    fn view_telemetry_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Telemetry");
    }

    /// Display packet inspector tab.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    fn view_packet_inspector_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Packet Inspector");
    }
}
