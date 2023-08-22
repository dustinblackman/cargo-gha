#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

mod add_asset;
mod binary;
mod cargo_toml;
mod cli;
mod filesystem;

use std::process;

use owo_colors::OwoColorize;

#[tokio::main]
async fn main() {
    let res = cli::run().await;
    // Only reached if cargo-gha code fails, otherwise process exits early from
    // within binary::run.
    if let Err(res) = res {
        eprintln!("{}", format!("cargo-gha failed: {res}").red());
        process::exit(1);
    }
}
