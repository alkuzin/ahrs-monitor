// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! AHRS monitor entry point.

use ahrs_monitor;
use log;

fn main() {
    ahrs_monitor::init();

    if let Err(e) = ahrs_monitor::run() {
        log::error!("{}", e);
        std::process::exit(1);
    }
}
