use crate::config::{TarBinaryDefinition, TarMemberDefinition};
use crate::download::{download_to_tempfile, is_download_necessary};
use crate::vars::current_machine_vars;
use log::{debug, info};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{fs, io};
use url::Url;

pub fn process_tar_definition(
    source_url: &str,
    definition: &TarBinaryDefinition,
    target_dir: &Path,
    _skip_checksum: bool,
    dry_run: bool,
) -> Result<(), String> {
    let interpolated_url_str = strfmt::strfmt(source_url, &current_machine_vars())
        .expect("template URL should interpolate correctly");
    let interpolated_url =
        Url::parse(interpolated_url_str.as_str()).expect("template URL should be valid");
    let members_to_update: Vec<&TarMemberDefinition> = definition
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

    let tar_temp_path = dirs::cache_dir()
        .expect("should be able to find cache directory")
        .join("bin-manager")
        .join(
            interpolated_url
                .path_segments()
                .unwrap()
                .next_back()
                .unwrap(),
        );

    fs::create_dir_all(tar_temp_path.parent().unwrap()).map_err(|e| e.to_string())?;

    debug!("Fetching tar from {:?} into {:?}", interpolated_url_str, tar_temp_path);

    let tar_file = download_to_tempfile(&interpolated_url_str, &tar_temp_path)?;
    let stream: Box<dyn Read> = match tar_temp_path.extension().unwrap().to_str().unwrap() {
        "gz" => Box::new(flate2::read::GzDecoder::new(tar_file)),
        "bz2" => Box::new(bzip2::read::BzDecoder::new(tar_file)),
        "xz" => Box::new(xz::read::XzDecoder::new(tar_file)),
        _ => todo!("unimplemented"),
    };
    let mut tar_archive = tar::Archive::new(stream);

    for entry in tar_archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let entry_path: PathBuf = entry
            .path()
            .expect("tar entry path should be valid")
            .components()
            .skip(definition.strip_components as usize)
            .collect();

        debug!("tar member: {entry_path:?}");

        let dest_path = target_dir.join(&entry_path);
        if let Some(member) = members_to_update.iter().find(|m| m.dest == entry_path) {
            let dest_dir = dest_path.parent().unwrap();
            fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;

            let mut dest_file =
                fs::File::create(&dest_path).expect("destination file should open successfully");

            info!(
                "will extract '{}' into '{}' with permissions {:o}{}",
                entry_path.to_str().unwrap(),
                dest_path.to_str().unwrap(),
                member.perms,
                if dry_run { " -- dry run, skipping" } else { "" }
            );

            if !dry_run {
                io::copy(&mut entry, &mut dest_file).map_err(|e| e.to_string())?;

                fs::set_permissions(&dest_path, fs::Permissions::from_mode(member.perms))
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}
