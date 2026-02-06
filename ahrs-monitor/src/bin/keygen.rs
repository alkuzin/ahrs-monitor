// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

//! Keys generating tool.

use chrono::Local;
use rand::RngCore;
use std::{fs, path::Path};

fn main() {
    let secrets_dir = Path::new("configs/secrets");
    let aes_path = secrets_dir.join("aes.key");
    let hmac_path = secrets_dir.join("hmac.key");

    println!("Running Encryption Keys Generator");

    fs::create_dir_all(secrets_dir)
        .expect("Failed to create secrets directory");

    // Generating new keys.
    let mut aes_key = [0u8; 16];
    let mut hmac_key = [0u8; 32];
    let mut rng = rand::rng();

    rng.fill_bytes(&mut aes_key);
    rng.fill_bytes(&mut hmac_key);

    // Rewriting old keys.
    fs::write(&aes_path, aes_key).expect("Failed to write AES key");
    fs::write(&hmac_path, hmac_key).expect("Failed to write HMAC key");

    let now = Local::now().format("%m/%d/%Y %H:%M:%S").to_string();

    println!("Keys successfully updated in {secrets_dir:?} at {now}");
    println!("(Both firmware and AHRS Monitor MUST be recompiled)");
}
