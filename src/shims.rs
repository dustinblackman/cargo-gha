#[cfg(target_family = "unix")]
use std::env;
use std::fs;
use std::io::Write;
use std::path;

use anyhow::Result;

use crate::cargo_toml;
use crate::filesystem;

#[cfg(target_family = "unix")]
fn create_shim(binary: &str, bin_path: path::PathBuf) -> Result<()> {
    use std::os::unix::prelude::OpenOptionsExt;

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

    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o770)
        .open(bin_path)?;

    write!(f, "{}", script)?;

    return Ok(());
}

#[cfg(not(target_family = "unix"))]
fn create_shim(binary: &str, bin_path: path::PathBuf) -> Result<()> {
    let script = format!(
        r#"@echo off
cargo gha {binary} %*
"#
    );

    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(bin_path)?;

    write!(f, "{}", script)?;

    return Ok(());
}

pub fn create(assets: Vec<cargo_toml::Asset>) -> Result<()> {
    let bin_dir = filesystem::get_project_root()?.join(".gha/.shims");
    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir)?;
    }

    for asset in assets {
        for binary in asset.binaries {
            let mut bin_path = bin_dir.join(&binary);

            bin_path.set_extension("");
            #[cfg(target_family = "windows")]
            {
                bin_path.set_extension("cmd");
            }
            if bin_path.exists() {
                continue;
            }

            if bin_path.exists() {
                continue;
            }

            create_shim(&binary, bin_path)?;
        }
    }

    return Ok(());
}
