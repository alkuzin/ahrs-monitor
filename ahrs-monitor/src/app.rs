// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application handler related declarations.

use std::collections::VecDeque;
use eframe::Frame;
use egui::{Align, CentralPanel, Color32, Context, Layout, RichText, TopBottomPanel};
use tokio::sync::mpsc::{Receiver};
use crate::{config, model::{AppEvent, FrameContext}, ui::{inspector, Tab}};

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
    /// History of the last N frame contexts.
    history: VecDeque<FrameContext>,
    /// Current frame context.
    current_frame: Option<FrameContext>,
    /// Indicator whether UI is paused.
    is_paused: bool,
}

impl eframe::App for App {
    /// Repaint UI.
    ///
    /// # Parameters
    /// - `ctx` - given egui context to handle.
    /// - `frame` - given surroundings of the app.
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| self.display_top_panel(ui));
        CentralPanel::default().show(ctx, |ui| self.display_central_panel(ui) );
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| self.display_bottom_panel(ui, ctx));

        self.handle_events();
        self.frame_counter += 1;
        // ctx.request_repaint();
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
    #[must_use]
    pub fn new(rx: Receiver<AppEvent>) -> Self {
        Self {
            rx,
            current_tab: Tab::default(),
            fps: 0.0,
            frame_counter: 0,
            connection_status: false,
            history: VecDeque::with_capacity(config::HISTORY_MAX_SIZE),
            is_paused: false,
            current_frame: None,
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
    #[allow(clippy::cast_possible_truncation)]
    fn fps(&mut self, ctx: &Context) -> usize {
        let current_fps = 1.0 / ctx.input(|i| f64::from(i.stable_dt));

        // Smoothing coefficient.
        let alpha = 0.1;

        if self.frame_counter <= 1 {
            self.fps = current_fps;
        }
        else {
            self.fps = self.fps + alpha * (current_fps - self.fps);
        }

        #[allow(clippy::cast_sign_loss)]
        {
            self.fps.max(0.0).round() as usize
        }
    }

    /// Display top panel.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    fn display_top_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // TODO: get titles from tabs trait method`.
            ui.selectable_value(&mut self.current_tab, Tab::Dashboard, "🗖 Dashboard");
            ui.selectable_value(&mut self.current_tab, Tab::Telemetry, "📈 Telemetry");
            ui.selectable_value(&mut self.current_tab, Tab::Inspector, "🔍 Packet Inspector");
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.is_paused, "⏸ Pause Stream");

            if self.is_paused {
                ui.label(RichText::new("(PAUSED)").color(Color32::YELLOW));
            }
        });
    }

    /// Display central panel.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    fn display_central_panel(&self, ui: &mut egui::Ui) {
        match self.current_tab {
            // TODO: add trait for Tabs.
            Tab::Inspector => inspector::display_tab(ui, &self.current_frame),
            Tab::Dashboard => Self::view_dashboard_tab(ui),
            Tab::Telemetry => Self::view_telemetry_tab(ui),
        }
    }

    /// Display bottom panel.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    /// - `ctx` - given egui context to handle.
    fn display_bottom_panel(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.horizontal(|ui| {
            let connection_label = if self.connection_status {
                RichText::new("CONNECTED").color(Color32::GREEN)
            }
            else {
                RichText::new("DISCONNECTED").color(Color32::RED)
            };

            ui.label(connection_label);
            ui.separator();

            if let Some(frame_ctx) = &self.current_frame {
                ui.label(format!("Total packets: {}", frame_ctx.total_packets));
                ui.separator();
                ui.label(format!("Bad packets: {}", frame_ctx.bad_packets));
                ui.separator();
                ui.label(format!("Stream: {} packets/sec", frame_ctx.pps));
                ui.separator();
            }

            // Colored FPS indicator.
            let current_fps = self.fps(ctx);

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
            ui.separator();

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(format!("AHRS Monitor - v{}", config::VERSION));
            });
        });
    }

    /// Handle events from ingester.
    fn handle_events(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                AppEvent::UpdateConnectionStatus(status) => {
                    self.connection_status = status;
                },
                AppEvent::FrameReceived(frame_ctx) => {
                    self.history.push_back(*frame_ctx.clone());

                    if self.history.len() > config::HISTORY_MAX_SIZE {
                        self.history.pop_front();
                    }

                    if !self.is_paused {
                        self.current_frame = Some(*frame_ctx);
                    }
                },
            }
        }
    }

    /// Display dashboard tab.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    fn view_dashboard_tab(ui: &mut egui::Ui) {
        ui.label("Dashboard");
    }

    /// Display telemetry tab.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    fn view_telemetry_tab(ui: &mut egui::Ui) {
        ui.label("Telemetry");
    }
}
