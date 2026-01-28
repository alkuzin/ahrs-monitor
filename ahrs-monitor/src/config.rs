// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Application's configurations.

/// Window width in pixels.
pub const APP_WINDOW_WIDTH: f32 = 1024.0;

/// Window height in pixels.
pub const APP_WINDOW_HEIGHT: f32 = 768.0;

/// Window size in pixels.
pub const APP_WINDOW_SIZE: [f32; 2] = [APP_WINDOW_WIDTH, APP_WINDOW_HEIGHT];

/// Title of the window.
pub const APP_WINDOW_TITLE: &'static str = "AHRS Monitor";

/// Project version.
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Ingester UDP IP address.
pub const UDP_IP_ADDR: &'static str = "127.0.0.1";

/// Ingester UDP port.
pub const UDP_PORT: u16 = 10000;

/// Max size of frame contexts history.
pub const HISTORY_MAX_SIZE: usize = 32;

/// MPSC channel max number of messages in the buffer.
pub const MPSC_CHANNEL_BUFFER_SIZE: usize = 128;
