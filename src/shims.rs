use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::prelude::OpenOptionsExt;

use anyhow::Result;

use crate::cargo_toml;
use crate::filesystem;

fn create_shim(binary: &str) -> Result<String> {
    let shell = env::var("SHELL")
        .unwrap_or("bash".to_string())
        .split('/')
        .last()
        .unwrap()
        .to_string();

    let script = format!(
        r#"#!/usr/bin/env {shell}

if [ ! -t 0 ]; then
    cat - | cargo gha {binary} "$@"
else
    cargo gha {binary} "$@"
fi"#
    );

    return Ok(script);
}

pub fn create(assets: Vec<cargo_toml::Asset>) -> Result<()> {
    let bin_dir = filesystem::get_project_root()?.join(".gha/.shims");
    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir)?;
    }

    for asset in assets {
        for binary in asset.binaries {
            let script = create_shim(&binary)?;
            let bin_path = bin_dir.join(&binary);
            if bin_path.exists() {
                continue;
            }
            let mut f = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .mode(0o770)
                .open(&bin_path)?;

            write!(f, "{}", script)?;
        }
    }

    return Ok(());
}
