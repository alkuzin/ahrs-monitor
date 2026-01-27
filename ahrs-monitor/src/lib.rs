// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! AHRS cross-platform telemetry station.

pub mod app;
pub mod config;
pub mod core;

use eframe::{egui, Error, Frame, HardwareAcceleration};
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::{io::Write, sync::Once};
use crate::core::{Ingester};

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

            writeln!(buf, "[{}][{}][{}] {}", timestamp, level, target, args)
        });

        builder.init()
    });
}

/// Initialize AHRS monitor.
pub fn init() {
    init_logging(LevelFilter::Info);
    log::info!("Initialized AHRS monitor");
}

/// Run AHRS monitor.
///
/// # Returns
/// - `Ok`  - in case of success.
/// - `Err` - otherwise.
pub fn run() -> eframe::Result {
    // Spawning a new asynchronous task for handling IDTP frames.
    tokio::spawn(async move {
        let mut ingester = Ingester::new();

        if let Err(e) = ingester.run().await {
            log::error!("Core service failed: {:?}", e);
        }
    });

    // Setting options controlling the behavior of a native window.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(config::APP_WINDOW_SIZE),
        hardware_acceleration: HardwareAcceleration::Required,
        ..Default::default()
    };

    // Starting a native app.
    eframe::run_native(
        config::APP_WINDOW_TITLE,
        options,
        Box::new(|_| Ok(Box::<app::App>::default())),
    )
}
