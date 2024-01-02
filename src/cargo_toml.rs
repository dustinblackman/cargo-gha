use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use serde::Deserialize;
use toml_edit::Array;
use toml_edit::ArrayOfTables;
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

    let mut asset_doc = None;
    if doc.get("package").is_some() && doc["package"]["metadata"]["gha"].get("assets").is_some() {
        asset_doc = Some(doc["package"]["metadata"]["gha"]["assets"].clone());
    } else if doc.get("workspace").is_some()
        && doc["workspace"]["metadata"]["gha"].get("assets").is_some()
    {
        asset_doc = Some(doc["workspace"]["metadata"]["gha"]["assets"].clone());
    }

    let mut assets: Vec<Asset> = vec![];
    if let Some(asset_doc) = asset_doc {
        let mut tmp_doc = Document::new();
        tmp_doc["items"] = asset_doc;
        let asset_res: Result<HashMap<String, Vec<Asset>>, toml::de::Error> =
            toml::from_str(&tmp_doc.to_string());
        assets = asset_res?.get("items").unwrap().to_vec();
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

    let mut asset_table = Table::default();
    asset_table.insert("owner_repo", Item::Value(asset.owner_repo.clone().into()));
    asset_table.insert("tag", Item::Value(asset.tag.clone().into()));
    asset_table.insert(
        "binaries",
        Item::Value(Value::Array(Array::from_iter(
            asset.binaries.clone().into_iter(),
        ))),
    );

    let mut target_archives_table = InlineTable::default();

    for (target, archive) in asset.target_archives.clone().into_iter() {
        target_archives_table.insert(&target, archive.into());
    }

    asset_table.insert(
        "target_archives",
        Item::Value(Value::InlineTable(target_archives_table)),
    );

    if doc["package"]["metadata"]["gha"].get("assets").is_none() {
        doc["package"]["metadata"]["gha"]["assets"] = Item::ArrayOfTables(ArrayOfTables::new());
    }

    let assets = doc["package"]["metadata"]["gha"]["assets"]
        .as_array_of_tables_mut()
        .unwrap();
    assets.push(asset_table);

    fs::write(config_path, doc.to_string())?;

    return Ok(());
}
