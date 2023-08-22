use std::env;
use std::fs;
use std::os::unix::prelude::PermissionsExt;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Result;
use walkdir::WalkDir;

use crate::cargo_toml;

pub fn get_project_root() -> Result<PathBuf> {
    let path = env::current_dir()?;
    let path_ancestors = path.as_path().ancestors();

    for p in path_ancestors {
        let has_cargo = fs::read_dir(p)?.any(|p| return p.unwrap().file_name() == *"Cargo.lock");

        if has_cargo {
            return Ok(PathBuf::from(p));
        }
    }

    return Err(anyhow!("Root directory for rust project not found."));
}

pub fn get_asset_directory(asset: &cargo_toml::Asset) -> Result<PathBuf> {
    let asset_directory = get_project_root()?.join(format!(
        ".gha/{owner_repo}/{tag}/",
        owner_repo = asset.owner_repo,
        tag = asset.tag
    ));

    return Ok(asset_directory);
}

pub fn list_binaries(asset: &cargo_toml::Asset) -> Result<Vec<PathBuf>> {
    let archive_path = get_asset_directory(asset)?;
    let binaries = WalkDir::new(archive_path)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| return e.ok())
        .filter(|e| {
            let walked_path = e.path();
            return walked_path.is_file()
                && walked_path.extension().is_none()
                && walked_path.metadata().unwrap().permissions().mode() & 0o111 != 0;
        })
        .map(|e| return e.into_path())
        .collect::<Vec<PathBuf>>();

    return Ok(binaries);
}
