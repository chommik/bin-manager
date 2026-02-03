use crate::config::process_config_file;
use args::Args;
use clap::Parser;
use log::{debug, warn, LevelFilter};
use std::fs;
use std::path::{Path, PathBuf};

mod args;
mod config;
mod definitions;
mod download;
mod utils;
mod vars;

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let config_dir: PathBuf = args
        .config_dir
        .map(|d| d.into())
        .or(dirs::config_dir()
            .map(|default_config_dir| default_config_dir.leak().join(Path::new("binaries"))))
        .expect("config directory not given and auto-detection failed");

    let target_dir: PathBuf = args
        .target_dir
        .map(|d| d.into())
        .or(dirs::executable_dir())
        .expect("target directory not given and auto-detection failed");

    debug!("config directory: {:?}", config_dir);
    debug!("target directory: {:?}", target_dir);

    for entry in fs::read_dir(config_dir)? {
        let path = entry?.path();

        if path.is_dir() {
            warn!("{} is a directory, skipping", path.display());
            continue;
        }

        process_config_file(&path, &target_dir, args.skip_checksum, args.dry_run)
            .expect("config file should process successfully");
    }

    Ok(())
}
