// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application handler related declarations.

use eframe::Frame;
use egui::{Align, CentralPanel, Color32, Context, Layout, RichText, TopBottomPanel};
use tokio::sync::mpsc::{Receiver};
use crate::{config, model::AppEvent};

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
pub struct App {
    /// MPSC receiver handle.
    rx: Receiver<AppEvent>,
    /// Current selected tab.
    current_tab: Tab,
    /// Current smoothed number of frames per second.
    fps: f64,
    /// Current number of frames from the start.
    frame_counter: usize,
    /// IMU connection status.
    connection_status: bool,
}

impl eframe::App for App {
    /// Repaint UI.
    ///
    /// # Parameters
    /// - `ctx` - given egui context to handle.
    /// - `frame` - given surroundings of the app.
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let current_fps = self.fps(ctx);

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Dashboard, "🗖 Dashboard");
                ui.selectable_value(&mut self.current_tab, Tab::Telemetry, "📈 Telemetry");
                ui.selectable_value(&mut self.current_tab, Tab::Inspector, "🔍 Packet Inspector");
            })
        });

        // Handling events from ingester.
        if let Ok(event) = self.rx.try_recv() {
            log::info!("EVENT: {:?}", event);

            match event {
                AppEvent::UpdateConnectionStatus(status) => {
                    self.connection_status = status;
                },
                AppEvent::FrameReceived(frame_ctx) => {
                    log::debug!("received frame: {:?}", frame_ctx);
                },
            }
        }

        CentralPanel::default().show(&ctx, |ui| {
            match self.current_tab {
                Tab::Inspector => self.view_packet_inspector_tab(ui),
                Tab::Dashboard => self.view_dashboard_tab(ui),
                Tab::Telemetry => self.view_telemetry_tab(ui),
            }
        });

        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let connection_label = if self.connection_status {
                    RichText::new("CONNECTED").color(Color32::GREEN)
                }
                else {
                    RichText::new("DISCONNECTED").color(Color32::RED)
                };

                ui.label(connection_label);
                ui.separator();

                // Colored FPS indicator.
                let fps_color = match current_fps {
                    f if f >= 60 => Color32::from_rgb(0, 200, 0),
                    f if f >= 45 => Color32::from_rgb(120, 200, 0),
                    f if f >= 30 => Color32::from_rgb(255, 165, 0),
                    f if f > 0 => Color32::from_rgb(220, 30, 30),
                    _ => Color32::GRAY,
                };

                let fps_label = RichText::new(
                    format!("FPS: {current_fps}")
                ).color(fps_color);
                ui.label(fps_label);

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(format!("AHRS Monitor - v{}", config::VERSION));
                });
            })
        });

        self.frame_counter += 1;
    }
}

impl App {
    /// Construct new `App` object.
    ///
    /// # Parameters
    /// - `rx` - given MPSC receiver handle.
    ///
    /// # Returns
    /// - New `App` object.
    pub fn new(rx: Receiver<AppEvent>) -> Self {
        Self {
            rx,
            current_tab: Default::default(),
            fps: 0.0,
            frame_counter: 0,
            connection_status: false,
        }
    }

    /// Get smoothed number of frames per second.
    /// (Exponential Moving Average (EMA)).
    ///
    /// # Parameters
    /// - `ctx` - given egui context to handle.
    ///
    /// # Returns
    /// - Smoothed number of frames per second.
    fn fps(&mut self, ctx: &Context) -> usize {
        let current_fps = 1.0 / ctx.input(|i| i.stable_dt as f64);

        // Smoothing coefficient.
        let alpha = 0.1;

        if self.frame_counter <= 1 {
            self.fps = current_fps;
        }
        else {
            self.fps = self.fps + alpha * (current_fps - self.fps)
        };

        self.fps as usize
    }

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
