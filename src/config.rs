use crate::definitions::direct::process_direct_definition;
use crate::definitions::tar::process_tar_definition;
use crate::vars::current_machine_vars;
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

    /// checksum key; defaults to target triplet
    pub key: Option<String>,

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
    #[serde(default = "default_perms")]
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
    pub source_url: Option<String>,
    pub source_urls: Option<HashMap<String, String>>,

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

    let current_machine_vars = current_machine_vars();
    let source_url =
        if let Some(source_url_template) = config.source_url {
            strfmt::strfmt(&source_url_template, &current_machine_vars)
                .expect("'source_url' format string should be correct")
        } else if let Some(source_url_template_map) = config.source_urls {
            let (source_url_key, source_url) = source_url_template_map.get_key_value(current_platform::CURRENT_PLATFORM)
                .or_else(|| source_url_template_map.get_key_value("{os}-{arch}"))
                .or_else(|| source_url_template_map.get_key_value("{os}-{cpu}"))
                .or_else(|| source_url_template_map.get_key_value("_any_"))
                .expect("'source_url_map' should contain a choice for current machine");

            debug!("Matched key '{}' from source_urls ('{}')", source_url_key, source_url);
            source_url.clone()
        } else { panic!("neither source_url nor source_urls given") };

    match config.definition {
        BinaryDefinition::Direct(def) => {
            process_direct_definition(&source_url, &def, target_dir, skip_checksum, dry_run)
        }
        BinaryDefinition::Tar(def) => {
            process_tar_definition(&source_url, &def, target_dir, skip_checksum, dry_run)
        }
    }?;

    Ok(())
}
