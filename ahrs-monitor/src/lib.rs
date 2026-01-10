// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! AHRS cross-platform telemetry station.

use eframe::{egui, Error, Frame};
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::{io::Write, sync::Once};

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
pub fn run() -> eframe::Result {
    Ok(())
}
