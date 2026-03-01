// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU simulator entry point.

mod simulator;
mod utils;

use log::LevelFilter;
use crate::simulator::ImuSimulator;
use ahrs_monitor::config::{self, load_config};
use ahrs_monitor::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("TEST");
    init_logging(LevelFilter::Info);
    log::info!("Initialized IMU simulator");

    if let Err(e) = run_simulator().await {
        eprintln!("Simulator exited unexpectedly: {e}");
    }

    Ok(())
}

async fn run_simulator() -> anyhow::Result<()> {
    println!("TEST");
    let app_config = load_config(config::CONFIG_FILE_PATH);

    println!("TEST");
    println!("Setting simulator...");
    let imu_simulator = ImuSimulator::new(app_config);

    println!("Simulating IMU data transmission over UDP");
    imu_simulator.simulate_udp_transmission().await?;

    Ok(())
}
