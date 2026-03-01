// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application handler related declarations.

use crate::{
    config,
    config::AppConfig,
    core::StandardPayload,
    logger::{LogRecord, Logger, ToLog},
    model::{AppEvent, FrameContext},
    ui::{AppTab, DashboardTab, InspectorTab, TabViewer},
};
use eframe::Frame;
use egui::{
    Align, CentralPanel, Color32, Context, Layout, RichText, TopBottomPanel,
};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::mpsc::Receiver;

/// Application handler.
pub struct App {
    /// Given global config.
    config: AppConfig,
    /// MPSC receiver handle.
    rx: Receiver<AppEvent>,
    /// List of application tabs.
    tabs: Vec<AppTab>,
    /// Current selected tab index.
    current_tab_idx: usize,
    /// Current smoothed number of frames per second.
    fps: f64,
    /// Current number of frames from the start.
    frame_counter: usize,
    /// IMU connection status.
    connection_status: bool,
    /// History of the last N frame contexts.
    history: VecDeque<Arc<FrameContext>>,
    /// Current frame context.
    current_frame: Option<Arc<FrameContext>>,
    /// Indicator whether UI is paused.
    is_paused: bool,
    /// IMU data logger.
    logger: Option<Logger>,
}

impl eframe::App for App {
    /// Repaint UI.
    ///
    /// # Parameters
    /// - `ctx` - given egui context to handle.
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        TopBottomPanel::top("top_panel")
            .show(ctx, |ui| self.display_top_panel(ui));

        CentralPanel::default().show(ctx, |ui| self.display_central_panel(ui));

        TopBottomPanel::bottom("bottom_panel")
            .show(ctx, |ui| self.display_bottom_panel(ui, ctx));

        self.handle_events();
        self.frame_counter += 1;
    }
}

impl App {
    /// Construct new `App` object.
    ///
    /// # Parameters
    /// - `config` - given global config.
    /// - `rx` - given MPSC receiver handle.
    ///
    /// # Returns
    /// - New `App` object.
    #[must_use]
    pub fn new(config: AppConfig, rx: Receiver<AppEvent>) -> Self {
        Self {
            config,
            rx,
            fps: 0.0,
            frame_counter: 0,
            connection_status: false,
            history: VecDeque::with_capacity(config::HISTORY_MAX_SIZE),
            is_paused: false,
            current_frame: None,
            tabs: vec![
                AppTab::Dashboard(DashboardTab::default()),
                AppTab::Telemetry(Box::default()),
                AppTab::Inspector(InspectorTab),
            ],
            current_tab_idx: 0,
            logger: None,
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
        } else {
            self.fps = self.fps + alpha * (current_fps - self.fps);
        }

        #[allow(clippy::cast_sign_loss)]
        {
            self.fps.max(0.0).round() as usize
        }
    }

    /// Enable/disable IMU data logging.
    #[inline]
    pub fn toggle_logging(&mut self) {
        if self.logger.is_some() {
            self.logger = None;
        } else {
            self.logger = Logger::new(&self.config).ok();
        }
    }

    /// Display top panel.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    fn display_top_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (index, tab) in self.tabs.iter().enumerate() {
                let (icon, title) = match tab {
                    AppTab::Dashboard(tab) => (tab.icon(), tab.title()),
                    AppTab::Telemetry(tab) => (tab.icon(), tab.title()),
                    AppTab::Inspector(tab) => (tab.icon(), tab.title()),
                };

                let tab_label = format!("{icon} {title}");
                let checked = self.current_tab_idx == index;

                if ui.selectable_label(checked, tab_label).clicked() {
                    self.current_tab_idx = index;
                }
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            self.display_pause_button(ui);
            self.display_record_button(ui);

            if self.logger.is_some() && self.is_paused {
                ui.label("⚠ Warning: Interface paused, but logging is ACTIVE");
            }
        });
    }

    /// Display pause button.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    fn display_pause_button(&mut self, ui: &mut egui::Ui) {
        let pause_color = if self.is_paused {
            Color32::from_rgb(255, 165, 0)
        } else {
            Color32::from_gray(60)
        };

        let text = if self.is_paused {
            "▶ Resume Stream"
        } else {
            "⏸ Pause Stream"
        };

        let btn =
            egui::Button::new(RichText::new(text).strong()).fill(pause_color);

        if ui.add(btn).clicked() {
            self.is_paused = !self.is_paused;
        }

        if self.is_paused {
            ui.label(
                RichText::new("(DISPLAY FROZEN)")
                    .color(Color32::YELLOW)
                    .small(),
            );
        }
    }

    /// Display record button.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    fn display_record_button(&mut self, ui: &mut egui::Ui) {
        let is_logging = self.logger.is_some();

        let (btn_label, btn_color) = {
            self.logger.as_ref().map_or_else(
                || ("⏺ Record".to_string(), Color32::from_gray(40)),
                |logger| {
                    (format!("⏹ {}", logger.timestamp_str()), Color32::DARK_RED)
                },
            )
        };

        let response = ui.add(egui::Button::new(btn_label).fill(btn_color));

        if response.clicked() {
            self.toggle_logging();
        }

        let on_hover_text = if is_logging {
            "Stop Recording"
        } else {
            "Start Recording"
        };

        response.on_hover_text(on_hover_text);
    }

    /// Display central panel.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    #[inline]
    fn display_central_panel(&mut self, ui: &mut egui::Ui) {
        self.render_active_tab(ui);

        if !self.is_paused {
            ui.ctx().request_repaint();
        }
    }

    /// Display bottom panel.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    /// - `ctx` - given egui context to handle.
    fn display_bottom_panel(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.horizontal(|ui| {
            // Connection status label.
            let connection_label = if self.connection_status {
                RichText::new("CONNECTED").color(Color32::GREEN)
            } else {
                RichText::new("DISCONNECTED").color(Color32::RED)
            };

            ui.label(connection_label);
            ui.separator();

            // Encryption status label.
            let encryption_label = if self.config.net.use_encryption {
                RichText::new("🔒 ENCRYPTED").color(Color32::LIGHT_GREEN)
            } else {
                RichText::new("🔓 UNENCRYPTED").color(Color32::YELLOW)
            };

            ui.label(encryption_label);
            ui.separator();

            // Received packets info label.
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

            let fps_label =
                RichText::new(format!("FPS: {current_fps}")).color(fps_color);
            ui.label(fps_label);
            ui.separator();

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(format!("AHRS Monitor - v{}", config::VERSION));
            });
        });
    }

    /// Render active tab.
    ///
    /// # Parameters
    /// - `ui` - given screen UI handler.
    fn render_active_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(tab) = self.tabs.get_mut(self.current_tab_idx)
            && let Some(frame_ctx) = &self.current_frame
        {
            if self.config.imu.is_correct() {
                match tab {
                    AppTab::Dashboard(tab) => {
                        tab.ui(ui, frame_ctx, &self.config);
                    }
                    AppTab::Telemetry(tab) => {
                        tab.ui(ui, frame_ctx, &self.config);
                    }
                    AppTab::Inspector(tab) => {
                        tab.ui(ui, frame_ctx, &self.config);
                    }
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 2.0 - 50.0);
                    ui.add_space(10.0);

                    let label_text = "UNSUPPORTED IMU PAYLOAD TYPE";
                    ui.label(RichText::new(label_text).size(18.0));
                });
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 50.0);
                ui.add(egui::Spinner::new().size(40.0));
                ui.add_space(10.0);

                let label_text = if self.connection_status {
                    "WAITING FOR IMU DATA..."
                } else {
                    "WAITING FOR CONNECTION..."
                };

                ui.label(RichText::new(label_text).size(18.0));
            });
        }
    }

    /// Handle events from ingester.
    fn handle_events(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                AppEvent::UpdateConnectionStatus(status) => {
                    self.handle_update_connection_status(status);
                }
                AppEvent::FrameReceived(frame_ctx) => {
                    self.handle_received_frame(frame_ctx);
                }
            }
        }
    }

    /// Handle updating connection status event.
    ///
    /// # Parameters
    /// - `status` - given new connection status between AHRS monitor and IMU.
    #[inline]
    const fn handle_update_connection_status(&mut self, status: bool) {
        self.connection_status = status;
    }

    /// Handle received frame event.
    ///
    /// # Parameters
    /// - `frame_ctx` - given new frame context info.
    fn handle_received_frame(&mut self, frame_ctx: Box<FrameContext>) {
        let shared_ctx = Arc::new(*frame_ctx);

        self.history.push_back(Arc::clone(&shared_ctx));

        if self.history.len() > config::HISTORY_MAX_SIZE {
            self.history.pop_front();
        }

        if !self.is_paused {
            self.current_frame = Some(Arc::clone(&shared_ctx));

            if let Some(ref frame) = shared_ctx.frame {
                if let Some(AppTab::Telemetry(tab)) = self
                    .tabs
                    .iter_mut()
                    .find(|tab| matches!(tab, AppTab::Telemetry(_)))
                {
                    tab.add_data(frame);
                }

                if let Some(AppTab::Dashboard(tab)) = self
                    .tabs
                    .iter_mut()
                    .find(|tab| matches!(tab, AppTab::Dashboard(_)))
                {
                    tab.add_data(&shared_ctx.quaternion);
                }
            }

            self.write_record(&shared_ctx);
        }
    }

    /// Write record into file.
    ///
    /// # Parameters
    /// - `frame_ctx` - given current frame context info.
    fn write_record(&mut self, frame_ctx: &FrameContext) {
        if let Some(frame) = &frame_ctx.frame {
            let header = frame.header;

            let (q_w, q_x, q_y, q_z, roll, pitch, yaw) = frame_ctx
                .quaternion
                .map_or((1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0), |quat| {
                    let e = quat.euler_angles();
                    (quat.w, quat.i, quat.j, quat.k, e.0, e.1, e.2)
                });
            
            let mut record = LogRecord {
                timestamp: frame_ctx.timestamp,
                device_id: header.device_id,
                q_w,
                q_x,
                q_y,
                q_z,
                roll,
                pitch,
                yaw,
                ..LogRecord::default()
            };

            if let Some(payload) = &frame.payload {
                match payload {
                    StandardPayload::Imu3Acc(p) => p.fill_record(&mut record),
                    StandardPayload::Imu3Gyr(p) => p.fill_record(&mut record),
                    StandardPayload::Imu3Mag(p) => p.fill_record(&mut record),
                    StandardPayload::Imu6(p) => p.fill_record(&mut record),
                    StandardPayload::Imu9(p) => p.fill_record(&mut record),
                    StandardPayload::Imu10(p) => p.fill_record(&mut record),
                    StandardPayload::ImuQuat(p) => p.fill_record(&mut record),
                }
            }

            if let Some(logger) = &mut self.logger {
                logger.write(&record).ok();
            }
        }
    }
}
