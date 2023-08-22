use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use serde::Deserialize;
use toml_edit::value;
use toml_edit::Array;
use toml_edit::Document;
use toml_edit::InlineTable;
use toml_edit::Item;
use toml_edit::Table;
use toml_edit::Value;

use crate::filesystem;

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Asset {
    pub owner_repo: String,
    pub tag: String,
    pub binaries: Vec<String>,
    pub target_archives: HashMap<String, String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct MetadataValue {
    pub targets: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Metadata {
    pub assets: Vec<Asset>,
    pub targets: Vec<String>,
}

pub fn deserailize_metadata() -> Result<Metadata> {
    let toml_str: String =
        fs::read_to_string(filesystem::get_project_root()?.join("Cargo.toml"))?.parse()?;
    let doc = toml_str.parse::<Document>()?;

    let metadata_str = doc["package"]["metadata"]["gha"].to_string();
    let metadata_res: Result<MetadataValue, toml::de::Error> = toml::from_str(&metadata_str);
    let metadata = metadata_res?;

    let mut assets: Vec<Asset> = vec![];
    if doc["package"]["metadata"]["gha"].get("assets").is_some() {
        let asset_str = doc["package"]["metadata"]["gha"]["assets"].to_string();
        let asset_res: Result<HashMap<String, Asset>, toml::de::Error> = toml::from_str(&asset_str);
        assets = asset_res?
            .values()
            .map(|e| return e.clone())
            .collect::<Vec<Asset>>();
    }

    return Ok(Metadata {
        assets,
        targets: metadata.targets,
    });
}

pub fn add_asset(asset: &Asset) -> Result<()> {
    let config_path = filesystem::get_project_root()?.join("Cargo.toml");
    let toml_str: String = fs::read_to_string(&config_path)?.parse()?;
    let mut doc = toml_str.parse::<Document>()?;

    let owner_repo = &asset.owner_repo.replace(['-', '/'], "_");

    let mut asset_table = InlineTable::default();
    asset_table.insert("tag", asset.tag.clone().into());
    asset_table.insert("owner_repo", asset.owner_repo.clone().into());
    asset_table.insert(
        "binaries",
        Array::from_iter(asset.binaries.clone().into_iter()).into(),
    );

    let mut target_archives_table = InlineTable::default();

    for (target, archive) in asset.target_archives.clone().into_iter() {
        target_archives_table.insert(&target, archive.into());
    }

    asset_table.insert("target_archives", target_archives_table.into());

    if doc["package"]["metadata"]["gha"].get("assets").is_none() {
        doc["package"]["metadata"]["gha"]["assets"] = Item::Table(Table::new());
    }

    doc["package"]["metadata"]["gha"]["assets"][owner_repo] =
        value(Value::InlineTable(asset_table));

    fs::write(config_path, doc.to_string())?;

    return Ok(());
}
