use std::collections::HashMap;
use std::process;

use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use octocrab::models::repos::Release;
use octocrab::Octocrab;
use owo_colors::OwoColorize;

use crate::alias;
use crate::binary;
use crate::cargo_toml;
use crate::filesystem;

fn init_ctrl_c_handler() {
    ctrlc::set_handler(move || {
        let term = dialoguer::console::Term::stderr();
        let _ = term.show_cursor();
        process::exit(1);
    })
    .expect("error initializing CTRL+C handler")
}

async fn get_releases(owner_repo: &str) -> Result<Vec<Release>> {
    let split = owner_repo.split('/').collect::<Vec<&str>>();

    let client = Octocrab::builder().build()?;
    let owner = split[0];
    let repo = split[1];

    let releases = client
        .repos(owner, repo)
        .releases()
        .list()
        .per_page(10)
        .page(1u32)
        .send()
        .await?
        .into_iter()
        .collect::<Vec<Release>>();

    return Ok(releases);
}

fn ask(question: String, options: Vec<String>) -> Result<usize> {
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .default(0)
        .items(&options)
        .interact_opt()?
        .unwrap();

    return Ok(idx);
}

pub async fn interactive_add(url: String) -> Result<()> {
    init_ctrl_c_handler();

    let targets = cargo_toml::deserailize_metadata()?.targets;

    let mut owner_repo = url.replace("https://github.com/", "");
    if owner_repo.ends_with('/') {
        owner_repo.remove(owner_repo.len() - 1);
    }

    let mut asset = cargo_toml::Asset {
        owner_repo: owner_repo.to_string(),
        tag: "".to_string(),
        binaries: vec![],
        target_archives: HashMap::new(),
    };

    let releases = get_releases(&owner_repo).await?;
    let tags = releases
        .iter()
        .map(|e| {
            return e.tag_name.to_string();
        })
        .collect::<Vec<String>>();

    let selected_tag_index = ask(
        format!("What version of {owner_repo} do you want to install?"),
        tags,
    )?;

    asset.tag = releases[selected_tag_index].tag_name.to_string();

    let release_assets = &releases[selected_tag_index]
        .assets
        .iter()
        .filter(|e| {
            return e.name.ends_with(".zip")
                || e.name.ends_with(".tar.gz")
                || e.name.ends_with(".tar.xz");
        })
        .map(|e| {
            return e.name.to_string();
        })
        .collect::<Vec<String>>();

    for target in targets {
        let asset_index = ask(
            format!("What assset contains binaries for target {target}?"),
            release_assets.to_vec(),
        )?;

        let asset_name = release_assets[asset_index]
            .to_string()
            .replace(&asset.tag, "{TAG}")
            .replace(&asset.tag.replace('v', ""), "{NOVTAG}");

        asset.target_archives.insert(target.to_string(), asset_name);
    }

    let download_url = binary::get_download_url(&asset).await?;
    eprintln!("{}", format!("Downloading {download_url}").yellow());
    binary::download(&asset).await?;

    asset.binaries = filesystem::list_binaries(&asset)?
        .iter()
        .map(|e| {
            return e.file_name().unwrap().to_str().unwrap().to_string();
        })
        .collect::<Vec<String>>();

    cargo_toml::add_asset(&asset)?;
    alias::create(vec![asset])?;

    eprintln!("{}", "Done!".green());
    return Ok(());
}
