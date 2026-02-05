// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! IMU simulator entry point.

mod simulator;
mod utils;

use crate::simulator::ImuSimulator;
use ahrs_monitor::config::{self, load_config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_config = load_config(config::CONFIG_FILE_PATH);

    println!("Setting simulator...");
    let imu_simulator = ImuSimulator::new(app_config);

    println!("Simulating IMU data transmission over UDP");
    imu_simulator.simulate_udp_transmission().await?;

    Ok(())
}
