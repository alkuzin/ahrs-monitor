// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-monitor project and contributors.

use rand::RngCore;
use std::{fs, path::Path};

fn main() {
    let secrets_dir = Path::new("configs/secrets");
    let aes_path = secrets_dir.join("aes.key");
    let hmac_path = secrets_dir.join("hmac.key");

    fs::create_dir_all(secrets_dir)
        .expect("Failed to create secrets directory");

    // Generating AES key.
    if !Path::new(&aes_path).exists() {
        let mut key = [0u8; 16];
        rand::rng().fill_bytes(&mut key);
        fs::write(aes_path, key).expect("Failed to write AES key");
    }

    // Generating HMAC key.
    if !Path::new(&hmac_path).exists() {
        let mut key = [0u8; 32];
        rand::rng().fill_bytes(&mut key);
        fs::write(hmac_path, key).expect("Failed to write HMAC key");
    }

    // Rebuild project in case of updating keys by using keygen.
    println!("cargo:rerun-if-changed=secrets/aes.key");
    println!("cargo:rerun-if-changed=secrets/hmac.key");
}
