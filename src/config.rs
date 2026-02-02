use crate::definitions::direct::process_direct_definition;
use crate::definitions::tar::process_tar_definition;
use log::{debug, info};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn default_checksum_key() -> String {
    "{os}-{arch}".into()
}

fn default_checksum_type() -> String {
    "sha256".into()
}

fn default_perms() -> u32 {
    0o755
}

#[derive(Deserialize, Debug)]
pub struct ChecksumDefinition {
    #[serde(rename = "type", default = "default_checksum_type")]
    checksum_type: String,

    #[serde(default = "default_checksum_key")]
    pub key: String,

    pub values: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct TarMemberDefinition {
    #[serde(rename = "name")]
    pub member_name: String,
    pub dest: String,
    #[serde(default = "default_perms")]
    pub perms: u32,
    pub checksum: ChecksumDefinition,
}

#[derive(Deserialize, Debug)]
pub struct DirectBinaryDefinition {
    pub dest: String,
    pub perms: u32,
    pub checksum: ChecksumDefinition,
}

#[derive(Deserialize, Debug)]
pub struct TarBinaryDefinition {
    pub strip_components: u8,
    pub members: Vec<TarMemberDefinition>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "action")]
pub enum BinaryDefinition {
    #[serde(rename = "direct")]
    Direct(DirectBinaryDefinition),

    #[serde(rename = "tar")]
    Tar(TarBinaryDefinition),
}

#[derive(Deserialize, Debug)]
pub struct BinaryConfig {
    pub source_url: String,

    #[serde(flatten)]
    pub definition: BinaryDefinition,
}

pub fn process_config_file(
    path: &Path,
    target_dir: &Path,
    skip_checksum: bool,
    dry_run: bool,
) -> Result<(), String> {
    info!("processing {}", path.to_str().unwrap());
    let config: BinaryConfig = toml::from_str(
        fs::read_to_string(path)
            .expect("failed to read config file")
            .as_str(),
    )
    .expect("failed to parse config file");
    debug!("config: {config:#?}");

    match config.definition {
        BinaryDefinition::Direct(def) => {
            process_direct_definition(&config.source_url, &def, target_dir, skip_checksum, dry_run)
        }
        BinaryDefinition::Tar(def) => {
            process_tar_definition(&config.source_url, &def, target_dir, skip_checksum, dry_run)
        }
    }?;

    Ok(())
}
