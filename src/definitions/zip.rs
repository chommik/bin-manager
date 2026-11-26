use crate::config::{ZipBinaryDefinition, ZipMemberDefinition};
use crate::download::{download_to_tempfile, is_download_necessary};
use crate::vars::current_machine_vars;
use log::{debug, info};
use std::{fs, io};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use url::Url;
use zip::ZipArchive;

pub fn process_zip_definition(
    source_url: &String,
    definition: &ZipBinaryDefinition,
    target_dir: &Path,
    dry_run: bool,
) -> Result<(), String> {
    let interpolated_url_str = strfmt::strfmt(source_url, &current_machine_vars())
        .expect("template URL should interpolate correctly");
    let interpolated_url =
        Url::parse(interpolated_url_str.as_str()).expect("template URL should be valid");

    let members_to_update: Vec<&ZipMemberDefinition> = definition
        .members
        .iter()
        .filter(|member| {
            let dest_path = Path::new(target_dir).join(Path::new(&member.dest));
            is_download_necessary(&dest_path, &member.checksum)
                .ok()
                .is_some_and(|r| r)
        })
        .collect();

    if members_to_update.is_empty() {
        info!("No members to update, skipping");
        return Ok(());
    }

    let zip_temp_path = dirs::cache_dir()
        .expect("shuold be able to find cache directory")
        .join("bin-manager")
        .join(
            interpolated_url
                .path_segments()
                .unwrap()
                .next_back()
                .unwrap(),
        );
    fs::create_dir_all(zip_temp_path.parent().unwrap()).map_err(|e| e.to_string())?;

    debug!(
        "Fetching zip from {:?} into {:?}",
        interpolated_url, zip_temp_path
    );

    let zip_file = download_to_tempfile(&interpolated_url_str, &zip_temp_path)?;
    let mut zip_archive = ZipArchive::new(zip_file).map_err(|e| e.to_string())?;

    let zip_entries: Vec<String> = zip_archive.file_names().filter_map(Result::ok).map(String::from).collect();
    for entry in zip_entries {
        debug!("zip member: {:?}", entry);
        let entry_path: PathBuf = Path::new(&entry.to_string())
            .components()
            .skip(definition.strip_components as usize)
            .collect();

        if let Some(member) = members_to_update.iter().find(|m| m.dest == entry_path) {
            let dest_path = target_dir.join(&entry_path);
            let dest_dir = dest_path.parent().unwrap();
            fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;

            info!(
                "Will extract '{} into {} with permissions {:o}{}",
                entry_path.to_str().unwrap(),
                dest_path.to_str().unwrap(),
                member.perms,
                if dry_run { " -- dry run, skipping" } else { "" }
            );

            if !dry_run {
                let mut dest_file =
                    fs::File::create(&dest_path).expect("destination file should open successfully");
                let mut zip_stream = zip_archive.by_name(&entry).expect("zip file member should exist");

                io::copy(&mut zip_stream, &mut dest_file).map_err(|e| e.to_string())?;
                fs::set_permissions(&dest_path, fs::Permissions::from_mode(member.perms)).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}
