use crate::config::DirectBinaryDefinition;
use crate::download::{download_to_destination, is_download_necessary};
use log::{debug, info};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn process_direct_definition(
    source_url: &str,
    definition: &DirectBinaryDefinition,
    target_dir: &Path,
    _skip_checksum: bool,
    dry_run: bool,
) -> Result<(), String> {
    let dest_path = Path::new(target_dir).join(Path::new(definition.dest.as_str()));

    let checksum = &definition.checksum;
    if !is_download_necessary(&dest_path, checksum)? {
        debug!("skipping re-download");
        return Ok(());
    }

    info!("downloading {} directly into {}{}",
        source_url,
        dest_path.to_str().unwrap(),
        if dry_run { " -- dry run; skipping" } else { "" }
    );

    if !dry_run {
        download_to_destination(source_url, &dest_path)?;
        fs::set_permissions(dest_path, fs::Permissions::from_mode(definition.perms))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}