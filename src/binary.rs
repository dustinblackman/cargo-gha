use std::env;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::path;
use std::process;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use async_compression::tokio::bufread::GzipDecoder;
use async_compression::tokio::bufread::XzDecoder;
use detect_targets::detect_targets;
use tokio::fs;
use tokio::io;
use tokio_stream::StreamExt;
use tokio_tar::Archive;
use tokio_util::io::StreamReader;

use crate::cargo_toml;
use crate::filesystem;

pub async fn get_download_url(asset: &cargo_toml::Asset) -> Result<String> {
    let target_archive = asset
        .target_archives
        .get(&detect_targets().await[0])
        .unwrap()
        .replace("{TAG}", &asset.tag)
        .replace("{NOVTAG}", &asset.tag.replace('v', ""));

    let url = format!(
        "https://github.com/{owner_repo}/releases/download/{tag}/{target_archive}",
        owner_repo = asset.owner_repo,
        tag = asset.tag
    );

    return Ok(url);
}

pub async fn download(asset: &cargo_toml::Asset) -> Result<path::PathBuf> {
    let url = get_download_url(asset).await?;
    let archive_res = reqwest::get(&url).await?;
    if !archive_res.status().is_success() {
        bail!("Failed to download: {url}");
    }

    let download_dir = filesystem::get_asset_directory(asset)?;
    fs::create_dir_all(&download_dir).await?;

    let mut archive_stream = archive_res.bytes_stream().map(|result| {
        return result.map_err(|error| return IoError::new(IoErrorKind::Other, error));
    });

    if url.ends_with(".tar.gz") {
        let stream_reader = StreamReader::new(archive_stream);
        Archive::new(GzipDecoder::new(stream_reader))
            .unpack(&download_dir)
            .await?;
    } else if url.ends_with(".tar.xz") {
        let stream_reader = StreamReader::new(archive_stream);
        Archive::new(XzDecoder::new(stream_reader))
            .unpack(&download_dir)
            .await?;
    } else if url.ends_with(".zip") {
        let temp_file_path = env::temp_dir().as_path().join("gha.zip");
        if temp_file_path.exists() {
            fs::remove_file(&temp_file_path).await?;
        }

        let mut file = fs::File::create(&temp_file_path).await?;
        while let Some(item) = archive_stream.next().await {
            io::copy(&mut item?.as_ref(), &mut file).await?;
        }
        file.sync_data().await?;
        // TODO all the async zip extractors are overly complex... Replace this later.
        zip::ZipArchive::new(std::fs::File::open(&temp_file_path).unwrap())?
            .extract(&download_dir)?;

        fs::remove_file(temp_file_path).await?;
    }

    return Ok(download_dir);
}

pub fn run(bin_path: path::PathBuf, args: Vec<String>) -> Result<()> {
    let mut system_shell_paths = env::var("PATH")
        .unwrap_or("".to_string())
        .split(':')
        .map(|e| return e.to_string())
        .collect::<Vec<String>>();

    let project_root = filesystem::get_project_root()?;
    let mut shell_paths = vec![project_root.join(".gha/.bin").to_string_lossy().to_string()];

    // https://github.com/dustinblackman/cargo-run-bin
    let runbin = project_root.join(".bin/.bin");
    if runbin.exists() {
        shell_paths.append(&mut vec![runbin.to_string_lossy().to_string()]);
    }

    shell_paths.append(&mut system_shell_paths);

    let spawn = process::Command::new(&bin_path)
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .stdin(process::Stdio::inherit())
        .args(&args)
        .env("PATH", shell_paths.join(":"))
        .spawn();

    if let Ok(mut spawn) = spawn {
        let status = spawn
            .wait()?
            .code()
            .ok_or_else(|| return anyhow!("Failed to get spawn exit code"))?;

        process::exit(status);
    }

    let bin_path_str = bin_path.to_str().unwrap();
    bail!(format!("Process failed to start: {bin_path_str}"));
}
