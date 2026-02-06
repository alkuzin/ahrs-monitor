// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! AHRS cross-platform telemetry station.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::todo,
    clippy::unreachable,
    missing_docs
)]

#[macro_use]
pub mod macros;
pub mod app;
pub mod config;
pub mod core;
pub mod logger;
pub mod model;
pub mod ui;

use crate::{app::App, config::AppConfig, core::Ingester, model::AppEvent};
use chrono::Local;
use eframe::{HardwareAcceleration, egui};
use env_logger::Builder;
use log::LevelFilter;
use std::{env, io::Write, sync::Once};
use tokio::sync::mpsc;

/// Used in order to ensure that the initialization code runs only once.
static INIT: Once = Once::new();

/// Initialize global logger.
///
/// # Parameters
/// - `filter` - given logger verbosity level filter to set.
fn init_logging(filter: LevelFilter) {
    INIT.call_once(|| {
        let mut builder = Builder::new();

        builder.filter(None, filter);
        builder.format(|buf, record| {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let target = record.target();
            let level = record.level();
            let args = record.args();

            writeln!(buf, "[{timestamp}][{level}][{target}] {args}")
        });

        builder.init();
    });
}

/// Initialize AHRS monitor.
fn init() -> AppConfig {
    init_logging(LevelFilter::Info);
    log::info!("Initialized AHRS monitor");

    let args: Vec<String> = env::args().collect();

    let config_path = args
        .iter()
        .position(|arg| arg == "--config")
        .and_then(|pos| args.get(pos + 1))
        .map_or(config::CONFIG_FILE_PATH, |s| s.as_str());

    log::info!("Loading configurations from: {config_path}");
    config::load_config(config_path)
}

/// Run AHRS monitor.
///
/// # Returns
/// - `Ok`  - in case of success.
/// - `Err` - otherwise.
///
/// # Errors
/// - Eframe errors.
pub fn run() -> eframe::Result {
    let app_config = init();
    let (tx, rx) = mpsc::channel::<AppEvent>(config::MPSC_CHANNEL_BUFFER_SIZE);
    let app_config_clone = app_config.clone();

    // Spawning a new asynchronous task for handling IDTP frames.
    tokio::spawn(async move {
        let mut ingester = Ingester::new(tx, app_config_clone.net);

        if let Err(e) = ingester.run().await {
            log::error!("Core service failed: {e:?}");
        }
    });

    // Setting options controlling the behavior of a native window.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_maximize_button(false)
            .with_inner_size(config::APP_WINDOW_SIZE),
        hardware_acceleration: HardwareAcceleration::Required,
        ..Default::default()
    };

    // Starting a native app.
    eframe::run_native(
        config::APP_WINDOW_TITLE,
        options,
        Box::new(|_| Ok(Box::new(App::new(app_config, rx)))),
    )
}
