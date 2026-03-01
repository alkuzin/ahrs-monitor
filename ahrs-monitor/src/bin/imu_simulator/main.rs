// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU simulator entry point.

mod simulator;
mod utils;

use crate::simulator::Simulator;
use ahrs_monitor::{
    config::{self, load_config},
    init_logging,
};
use log::LevelFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging(LevelFilter::Info);
    log::info!("Initialized IMU simulator");

    if let Err(e) = run_simulator().await {
        log::error!("Simulator exited unexpectedly: {e}");
    }

    Ok(())
}

/// Run IMU simulator.
///
/// # Returns
/// - `Ok` - in case of success.
/// - `Err` - otherwise.
///
/// # Errors
/// - I/O errors.
async fn run_simulator() -> anyhow::Result<()> {
    let app_config = load_config(config::CONFIG_FILE_PATH);

    log::info!("Setting simulator...");
    let mut sim = Simulator::new(app_config)?;

    log::info!("Simulating IMU data transmission over UDP");
    sim.simulate_udp_transmission().await?;
    Ok(())
}
