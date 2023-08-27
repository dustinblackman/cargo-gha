use std::env;

use anyhow::bail;
use anyhow::Result;
use clap::Arg;
use clap::ArgMatches;
use clap::Command;
use owo_colors::OwoColorize;

use crate::add_asset;
use crate::binary;
use crate::cargo_toml;
use crate::filesystem;
use crate::wrappers;

async fn run_binary(binary_name: String, args: Vec<String>) -> Result<()> {
    let assets = cargo_toml::deserailize_metadata()?.assets;
    let asset_res = assets.iter().find(|e| {
        return e.binaries.contains(&binary_name);
    });
    if asset_res.is_none() {
        bail!(format!("No asset found for binary {binary_name}"))
    }

    let asset = asset_res.unwrap();

    let mut binaries = filesystem::list_binaries(asset)?;
    let mut binary_absolute_path = binaries.iter().find(|e| {
        return *e.file_name().unwrap().to_str().unwrap() == binary_name;
    });
    if binary_absolute_path.is_none() {
        println!(
            "{}",
            format!(
                "Installing {owner_repo} {tag}",
                owner_repo = asset.owner_repo,
                tag = asset.tag
            )
            .yellow()
        );
        binary::download(asset).await?;
        binaries = filesystem::list_binaries(asset)?;
        binary_absolute_path = binaries.iter().find(|e| {
            return *e.file_name().unwrap().to_str().unwrap() == binary_name;
        });
    }

    if binary_absolute_path.is_none() {
        bail!(format!("Binary {binary_name} is missing in the installation. Review the contents in .gha and consider removing/adding the package"));
    }

    binary::run(binary_absolute_path.unwrap().to_path_buf(), args)?;

    return Ok(());
}

async fn install() -> Result<()> {
    let assets = cargo_toml::deserailize_metadata()?.assets;
    for asset in &assets {
        if !filesystem::get_asset_directory(asset)?.exists() {
            println!(
                "{}",
                format!(
                    "Installing {owner_repo} {tag}",
                    owner_repo = asset.owner_repo,
                    tag = asset.tag
                )
                .yellow()
            );
            binary::download(asset).await?;
        }
    }

    wrappers::create(assets)?;

    println!("{}", "Done!".green());
    return Ok(());
}

fn bool_arg_used(matches: &ArgMatches, arg_long: &str) -> bool {
    if let Some(used) = matches.get_one::<bool>(arg_long) {
        if *used {
            return true;
        }
    }

    if let Some(sub) = matches.subcommand_matches("gha") {
        if let Some(used) = sub.get_one::<bool>(arg_long) {
            if *used {
                return true;
            }
        }
    }

    return false;
}

fn str_arg_used(matches: &ArgMatches, arg_long: &str) -> Option<String> {
    if let Some(res) = matches.get_one::<String>(arg_long) {
        return Some(res.to_string());
    }

    if let Some(sub) = matches.subcommand_matches("gha") {
        if let Some(res) = sub.get_one::<String>(arg_long) {
            return Some(res.to_string());
        }
    }

    return None;
}

pub async fn run() -> Result<()> {
    let arg_add = Arg::new("add")
        .short('a')
        .long("add")
        .value_name("Github Repo URL")
        .num_args(1)
        .help("Add new asset to Cargo.toml");

    let arg_install = Arg::new("install")
        .short('i')
        .long("install")
        .num_args(0)
        .help("Install all configured artifacts, skips entries that are already installed.");

    let arg_help = Arg::new("help")
        .short('h')
        .long("help")
        .num_args(0)
        .help("Print help");

    let mut app = Command::new("cargo-gha")
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg_required_else_help(false)
        .ignore_errors(true)
        .arg(arg_add.clone())
        .arg(arg_install.clone())
        .subcommand(
            Command::new("gha")
                .hide(true)
                .disable_help_flag(true)
                .arg(arg_add)
                .arg(arg_install)
                .arg(arg_help),
        );

    let matches = app.clone().get_matches();

    if let Some(url) = str_arg_used(&matches, "add") {
        add_asset::interactive_add(url.to_string()).await?
    } else if bool_arg_used(&matches, "install") {
        install().await?;
    } else if bool_arg_used(&matches, "help") {
        app.print_long_help()?;
    } else {
        let mut args: Vec<_> = env::args().collect();
        let start_index = args.iter().position(|e| return e.ends_with("/cargo-gha"));
        if start_index.is_none() || start_index.unwrap() == (args.len() + 1) {
            app.print_long_help()?;
            return Ok(());
        }

        let mut bin_index = start_index.unwrap() + 1;
        if args[bin_index] == "gha" {
            bin_index += 1;
        }

        let binary_name = args[bin_index].clone();
        args.drain(0..(bin_index + 1));

        run_binary(binary_name, args).await?;
    }

    return Ok(());
}
